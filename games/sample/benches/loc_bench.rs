#![allow(unused)]

use analyzer::access::*;
use analyzer::context::*;
use analyzer::greedy::*;
use analyzer::world::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use enum_map::EnumMap;
use libsample::context::Context;
use libsample::graph;
use libsample::items::Item;

fn check_access_call(locs: &[graph::Location], ctx: &Context) -> i32 {
    let mut i = 0;
    for loc in locs {
        if loc.can_access(ctx) {
            i += 1
        }
    }
    i
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let locmap = graph::build_locations();
    let mut ctx = Context::default();
    ctx.collect(Item::Kokiri_Sword);
    ctx.collect(Item::Buy_Deku_Stick_1);

    c.bench_function("call", |b| {
        b.iter(|| check_access_call(locmap.as_slice(), &ctx))
    });

    let world = graph::World::new();
    let mut ctx = Context::new();
    world.skip_unused_items(&mut ctx);
    c.bench_function("can_win_from_scratch", |b| b.iter(|| can_win(&world, &ctx)));

    let ctx = ContextWrapper::new(Context::new());
    c.bench_function("greedy search", |b| b.iter(|| greedy_search(&world, &ctx)));
    c.bench_function("minimal playthrough", |b| {
        b.iter(|| minimal_greedy_playthrough(&world, &ctx))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(1000);
    targets = criterion_benchmark
}
criterion_main!(benches);
