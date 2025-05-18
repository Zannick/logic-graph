use analyzer::context::*;
use analyzer::estimates::ContextScorer;
use analyzer::route::*;
use analyzer::world::World;
use libsample::context;
use libsample::graph;
use libsample::items::Item;

#[test]
fn test_parse() {
    let world = graph::World::new();
    let startctx = context::Context::default();
    let scorer = ContextScorer::shortest_paths(&*world, &startctx, 32_768);

    let route = r#"
    * Collect Kokiri_Sword from KF > Boulder Maze > Reward > Chest
    ! Do KF > Kokiri Village > Mido's Porch > Gather Rupees
    ! Do KF > Kokiri Village > Mido's Porch > Gather Rupees
    * Collect Buy_Deku_Shield from KF > Shop > Entry > Item 1
    * Collect Showed_Mido from KF > Kokiri Village > Mido's Guardpost > Show Mido
    "#;

    let res = route_from_string(&*world, &startctx, route, scorer.get_algo());
    let ctx = res.unwrap();
    assert!(ctx.get().has(Item::Kokiri_Sword));
    assert!(ctx.get().has(Item::Buy_Deku_Shield));
    assert!(ctx.get().has(Item::Showed_Mido));
    assert_eq!(
        ctx.recent_history().last(),
        Some(&History::G(
            Item::Showed_Mido,
            graph::LocationId::KF__Kokiri_Village__Midos_Guardpost__Show_Mido
        ))
    )
}
