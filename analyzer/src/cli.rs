use crate::algo::Search;
use crate::context::*;
use crate::db::HeapDB;
use crate::route::*;
use crate::world::*;
use clap::{Parser, Subcommand};
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
            route_ctxs.extend(routes.into_iter().map(|route| {
                let rstr = read_from_file(route);
                route_from_string(world, &startctx, &rstr).unwrap()
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
            let rstr = read_from_file(route);
            println!(
                "{}",
                match debug_route(world, &startctx, &rstr) {
                    Ok(s) | Err(s) => s,
                }
            );
            Ok(())
        }
        Commands::Info => {
            println!(
                "data sizes: Context={} ContextWrapper={} serialized={} World={}\nstart overrides: {}\nobjective: {}",
                size_of::<T>(),
                size_of::<ContextWrapper<T>>(),
                HeapDB::<W, T>::serialize_state(&startctx).len(),
                size_of::<W>(),
                startctx.diff(&T::default()),
                world.objective_name()
            );
            Ok(())
        }
    }
}
