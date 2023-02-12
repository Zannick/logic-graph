use criterion::{black_box, criterion_group, criterion_main, Criterion};
use analyzer::context::Ctx;
use analyzer::world::*;
use libsample::context::Context;
use libsample::graph::{Location, build_locations};
use libsample::items::Item;
use enum_map::EnumMap;

fn check_access_call(locs: &[Location], ctx: &Context) -> i32 {
    let mut i = 0;
    for loc in locs {
        if loc.can_access(ctx) { i += 1 }
    }
    i
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let locmap = build_locations();
    let mut ctx = Context::default();
    ctx.collect(&Item::Kokiri_Sword);
    ctx.collect(&Item::Buy_Deku_Stick_1);

    c.bench_function("call", |b| b.iter(|| check_access_call(locmap.as_slice(), &ctx)));
}

criterion_group!{
    name = benches;
    config = Criterion::default().sample_size(1000);
    targets = criterion_benchmark
}
criterion_main!(benches);
