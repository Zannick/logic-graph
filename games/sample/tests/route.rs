use analyzer::estimates::ContextScorer;
use analyzer::route::*;
use analyzer::world::World;
use libsample::context;
use libsample::graph;

#[test]
fn test_parse() {
    let world = graph::World::new();
    let startctx = context::Context::default();

    let scorer = ContextScorer::shortest_paths(&*world, &startctx, 32_768);
    let res = route_from_string(&*world, &startctx, "", scorer.get_algo());
}
