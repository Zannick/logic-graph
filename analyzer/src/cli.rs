use crate::algo::Search;
use crate::context::*;
use crate::route::*;
use crate::world::*;
use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Graph algorithm analysis", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// searches for a solution
    Search {
        /// yaml file with settings and search parameters
        #[arg(short, long, value_name = "FILE")]
        settings: Option<PathBuf>,
        // TODO: move routes here into their own file(s)
    },

    /// evaluates a route and shows stepwise diffs
    Route {
        /// yaml file with settings
        #[arg(short, long, value_name = "FILE")]
        settings: Option<PathBuf>,

        /// text file with route
        #[arg(long, value_name = "FILE")]
        route: PathBuf,
    },
}

pub fn run<W, T, L>(
    world: &W,
    startctx: T,
    routes: Vec<ContextWrapper<T>>,
    args: &Cli,
) -> Result<(), std::io::Error>
where
    W: World<Location = L>,
    T: Ctx<World = W>,
    L: Location<Context = T>,
{
    match &args.command {
        Some(Commands::Search { .. }) => {
            let search = Search::new(world, startctx, routes)?;
            search.search()
        }
        Some(Commands::Route { route, .. }) => {
            let mut file = File::open(&route)
                .unwrap_or_else(|e| panic!("Couldn't open file \"{:?}\": {:?}", route, e));
            let mut rstr = String::new();
            file.read_to_string(&mut rstr)
                .unwrap_or_else(|e| panic!("Couldn't read from file \"{:?}\": {:?}", route, e));
            println!(
                "{}",
                match debug_route(world, &startctx, &rstr) {
                    Ok(s) | Err(s) => s,
                }
            );
            Ok(())
        }
        None => Ok(()),
    }
}
