use crate::algo::Search;
use crate::context::*;
use crate::db::HeapDB;
use crate::estimates::ContextScorer;
use crate::greedy::*;
use crate::matchertrie::MatcherTrie;
use crate::minimize::trie_minimize;
use crate::observer::record_observations;
use crate::observer::Observer;
use crate::route::*;
use crate::solutions::Solution;
use crate::world::*;
use clap::{Parser, Subcommand};
use similar::TextDiff;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Parser)]
#[command(about = "Graph algorithm analysis", long_about = None)]
pub struct Cli {
    /// yaml file with settings and search parameters
    #[arg(short, long, value_name = "FILE")]
    settings: Option<PathBuf>,

    #[arg(long, value_name = "FILE")]
    logconfig: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    pub fn settings_file(&self) -> Option<&PathBuf> {
        self.settings.as_ref()
    }

    pub fn logconfig(&self) -> Option<&PathBuf> {
        self.logconfig.as_ref()
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// searches for a solution
    Search {
        /// Text files with routes to start from
        #[arg(long, value_name = "FILE")]
        routes: Vec<PathBuf>,

        /// Directory in which to place the databases
        #[arg(long, value_name = "DIR")]
        db: Option<PathBuf>,
    },

    /// evaluates a route and shows stepwise diffs
    Route {
        /// text file with route
        #[arg(value_name = "FILE")]
        route: PathBuf,
    },

    /// performs a greedy search and exits
    Greedy {
        /// text file with route to start from
        #[arg(value_name = "FILE")]
        route: Option<PathBuf>,
    },

    /// Attempts to minimize the given route (must be a winning route)
    Minimize {
        /// test file with winning route
        #[arg(value_name = "FILE")]
        route: PathBuf,
    },

    /// provides debug info about the binary
    Info,
}

pub fn read_from_file<P>(p: &P) -> String
where
    P: AsRef<Path> + Debug,
{
    let mut file = File::open(p).unwrap_or_else(|e| panic!("Couldn't open file {:?}: {:?}", p, e));
    let mut rstr = String::new();
    file.read_to_string(&mut rstr)
        .unwrap_or_else(|e| panic!("Couldn't read from file {:?}: {:?}", p, e));
    rstr
}

pub fn run<W, T, L>(
    world: &W,
    startctx: T,
    mut route_ctxs: Vec<ContextWrapper<T>>,
    args: &Cli,
) -> Result<(), std::io::Error>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    log::info!("{:?}", std::env::args());

    match &args.command {
        Commands::Search { routes, db } => {
            // This duplicates the creation later by the heap wrapper.
            let scorer = ContextScorer::shortest_paths(world, &startctx, 32_768);

            route_ctxs.extend(routes.into_iter().map(|route| {
                let rstr = read_from_file(route);
                route_from_string(world, &startctx, &rstr, scorer.get_algo()).unwrap()
            }));
            let search = Search::new(
                world,
                startctx,
                route_ctxs,
                db.as_ref().unwrap_or(&".db".into()),
            )?;
            search.search()
        }
        Commands::Route { route, .. } => {
            // This duplicates the creation later by the heap wrapper.
            let scorer = ContextScorer::shortest_paths(world, &startctx, 32_768);
            let rstr = read_from_file(route);
            println!(
                "{}",
                match debug_route::<W, T, L, <W::Exit as Exit>::SpotId>(
                    world, &startctx, &rstr, &scorer
                ) {
                    Ok(s) | Err(s) => s,
                }
            );
            Ok(())
        }
        Commands::Greedy { route } => {
            // This duplicates the creation later by the heap wrapper.
            let scorer = ContextScorer::shortest_paths(world, &startctx, 32_768);
            let result = if let Some(r) = route {
                let ctx =
                    route_from_string(world, &startctx, &read_from_file(r), scorer.get_algo())
                        .unwrap();
                greedy_search(world, &ctx, u32::MAX, 2)
            } else {
                greedy_search_from(world, &startctx, u32::MAX)
            };
            if let Ok(found) = result {
                println!(
                    "{}",
                    history_summary::<T, _>(found.recent_history().iter().copied())
                );
            } else {
                println!("Could not find a greedy route");
            }
            Ok(())
        }
        Commands::Minimize { route } => {
            // This duplicates the creation later by the heap wrapper.
            let scorer = ContextScorer::shortest_paths(world, &startctx, 32_768);
            let ctx =
                route_from_string(world, &startctx, &read_from_file(route), scorer.get_algo())
                    .unwrap();
            if !world.won(ctx.get()) {
                println!("Route did not win");
                return Ok(());
            }
            let mut trie = MatcherTrie::<<T::Observer as Observer>::Matcher>::default();
            let solution = Arc::new(Solution {
                elapsed: ctx.elapsed(),
                history: ctx.recent_history().to_vec(),
            });
            record_observations(&startctx, world, solution.clone(), 0, None, &mut trie);
            println!(
                "Initial solution of length {} produces trie of size {} depth {} and num values {}",
                solution.history.len(),
                trie.size(),
                trie.max_depth(),
                trie.num_values()
            );
            if let Some(best) = trie_minimize(world, &startctx, solution.clone(), &trie) {
                println!(
                    "Improved route from {}ms to {}ms",
                    solution.elapsed,
                    best.elapsed()
                );
                let old_hist = history_str::<T, _>(solution.history.iter().copied());
                let new_hist = history_str::<T, _>(best.recent_history().iter().copied());
                let text_diff = TextDiff::from_lines(&old_hist, &new_hist);
                print!(
                    "{}",
                    text_diff
                        .unified_diff()
                        .context_radius(3)
                        .header("original", "best")
                );
            } else {
                println!("Could not improve solution.");
            }
            Ok(())
        }
        Commands::Info => {
            println!(
                "data sizes: Context={} ContextWrapper={} serialized={} World={}\nstart overrides: {}\nruleset: {}",
                size_of::<T>(),
                size_of::<ContextWrapper<T>>(),
                HeapDB::<W, T>::serialize_state(&startctx).len(),
                size_of::<W>(),
                startctx.diff(&T::default()),
                world.ruleset()
            );
            Ok(())
        }
    }
}
