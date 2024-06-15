use crate::search::Search;
use crate::context::*;
use crate::db::HeapDB;
use crate::estimates::ContextScorer;
use crate::greedy::*;
use crate::matchertrie::MatcherTrie;
use crate::minimize::{mutate_collection_steps, mutate_spot_revisits, trie_minimize};
use crate::observer::{debug_observations, record_observations, Observer};
use crate::route::*;
use crate::solutions::write_graph;
use crate::world::*;
use clap::{Parser, Subcommand};
use rustc_hash::FxHashSet;
use similar::TextDiff;
use std::fmt::Debug;
use std::fs::File;
use std::io::Read;
use std::mem::size_of;
use std::path::{Path, PathBuf};

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
        /// text file with winning route
        #[arg(value_name = "FILE")]
        route: PathBuf,
    },

    /// Creates a graph file of the given route (must be a winning route)
    Draw {
        /// text file with winning route
        #[arg(value_name = "FILE")]
        route: PathBuf,
    },

    /// Outputs debug info about observations between steps
    Observe {
        /// text file with winning route
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
                match route_from_string(world, &startctx, &rstr, scorer.get_algo()) {
                    Ok(r) => r,
                    Err((r, e)) => {
                        log::error!("Using partial route from {:?}: {}", route, e);
                        r
                    }
                }
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
            let mut ctx =
                route_from_string(world, &startctx, &read_from_file(route), scorer.get_algo())
                    .unwrap();
            if !world.won(ctx.get()) {
                let left = world.items_needed(ctx.get());
                println!("Route did not win: still need {:?}", left);
                return Ok(());
            }
            let mut trie = MatcherTrie::<<T::Observer as Observer>::Matcher>::default();
            let mut solution = ctx.to_solution();
            let orig = solution.clone();
            record_observations(&startctx, world, solution.clone(), 0, &mut trie);
            println!(
                "Initial solution ({}ms) of length {} produces trie of size {} depth {} and num values {}",
                solution.elapsed,
                solution.history.len(),
                trie.size(),
                trie.max_depth(),
                trie.num_values(),
            );
            let mut improvements = Vec::new();
            if let Some(better) = trie_minimize(world, &startctx, solution.clone(), &trie) {
                ctx = better;
                println!(
                    "Improved route via trie from {}ms to {}ms",
                    solution.elapsed,
                    ctx.elapsed()
                );
                improvements.push(ctx.clone());
                solution = ctx.to_solution();
            }

            let mut mutations = mutate_spot_revisits(world, &startctx, solution.clone());
            let old_len = mutations.len();
            mutations.retain(|c| world.won(c.get()));
            let mutations: Vec<_> = mutations.into_iter().map(|m| m.into_solution()).collect();
            if !mutations.is_empty() {
                let min = mutations.iter().min_by_key(|sol| sol.elapsed).unwrap();
                println!(
                    "Route swapping got {} solutions (best={}ms) and {} partials",
                    mutations.len(),
                    min.elapsed,
                    old_len - mutations.len()
                );
                for sol in &mutations {
                    record_observations(&startctx, world, sol.clone(), 0, &mut trie);
                }
                println!(
                    "After observing new routes, trie has: size {} depth {} and num values {}",
                    trie.size(),
                    trie.max_depth(),
                    trie.num_values(),
                );
                if let Some(better) = trie_minimize(world, &startctx, min.clone(), &trie) {
                    improvements.push(better);
                }
            }

            let mut reorders = 0;
            while let Some(reordered) = mutate_collection_steps(
                world,
                &startctx,
                solution.elapsed,
                2,
                2_048,
                solution.clone(),
                scorer.get_algo(),
            ) {
                reorders += 1;
                println!(
                    "Reorder got an improvement #{}: {}ms",
                    reorders,
                    reordered.elapsed()
                );
                solution = reordered.to_solution();
                improvements.push(reordered);
                record_observations(&startctx, world, solution.clone(), 0, &mut trie);
            }
            println!(
                "After observing new routes, trie has: size {} depth {} and num values {}",
                trie.size(),
                trie.max_depth(),
                trie.num_values(),
            );
            if let Some(better) = trie_minimize(world, &startctx, solution.clone(), &trie) {
                improvements.push(better);
            }

            if let Some(best) = improvements.into_iter().min_by_key(|c| c.elapsed()) {
                let old_hist = history_str::<T, _>(orig.history.iter().copied());
                let new_hist = history_str::<T, _>(best.recent_history().iter().copied());
                let text_diff = TextDiff::from_lines(&old_hist, &new_hist);
                print!(
                    "{}",
                    text_diff.unified_diff().context_radius(3).header(
                        &format!("original [{}ms]", orig.elapsed),
                        &format!(
                            "best [{}ms (-{}ms)]",
                            best.elapsed(),
                            orig.elapsed - best.elapsed()
                        )
                    )
                );
            } else {
                println!("Could not improve solution.");
            }
            Ok(())
        }
        Commands::Draw { route } => {
            // This duplicates the creation later by the heap wrapper.
            let scorer = ContextScorer::shortest_paths(world, &startctx, 32_768);
            let ctx =
                route_from_string(world, &startctx, &read_from_file(route), scorer.get_algo())
                    .unwrap();

            write_graph(world, &startctx, ctx.recent_history()).unwrap();
            Ok(())
        }
        Commands::Observe { route } => {
            let scorer = ContextScorer::shortest_paths(world, &startctx, 32_768);
            let ctx =
                route_from_string(world, &startctx, &read_from_file(route), scorer.get_algo())
                    .unwrap();
            if !world.won(ctx.get()) {
                let left = world.items_needed(ctx.get());
                println!("Route did not win: still need {:?}", left);
                return Ok(());
            }
            let solution = ctx.to_solution();
            debug_observations(&startctx, world, solution, 1);
            Ok(())
        }
        Commands::Info => {
            let items = world
                .unused_items()
                .into_iter()
                .map(|item| format!("{}", item))
                .collect::<Vec<_>>();
            let unskipped: Vec<_> = world
                .get_all_locations()
                .into_iter()
                .filter(|loc| !loc.skippable())
                .collect();
            let unskipped_len = unskipped.len();
            let canons: FxHashSet<_> = unskipped.into_iter().map(|loc| loc.canon_id()).collect();
            println!(
                "data sizes: Context={} ContextWrapper={} serialized={} World={}\nstart overrides: {}\nruleset: {}\n\
                unused items: ({}) {}\nLocations: total={}, unskipped={}, max visitable={}, max unskipped visitable={}\n",
                size_of::<T>(),
                size_of::<ContextWrapper<T>>(),
                HeapDB::<W, T>::serialize_state(&startctx).len(),
                size_of::<W>(),
                startctx.diff(&T::default()),
                world.ruleset(),
                items.len(),
                items.join(", "),
                world.get_all_locations().len(),
                unskipped_len,
                W::NUM_CANON_LOCATIONS,
                canons.len(),
            );
            Ok(())
        }
    }
}
