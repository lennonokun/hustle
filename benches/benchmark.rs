use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkGroup};
use criterion::measurement::Measurement;
use hustle::solve::{State, SData, AData, Cache};
use hustle::util::*;
use std::time::{Instant, Duration};

pub fn solve_bench(c: &mut Criterion) {
  fn measure(state: &State, sd: &SData) -> Duration {
    let sd = sd.deep_clone();
    let start = Instant::now();
    black_box(&state).solve_given(Word::from_str("SALET").unwrap(), &sd, u32::MAX);
    start.elapsed()
  }

  let state1 = State::new3();
  let mut state2 = State::new3();
  state2.hard = true;

  let sd_300_3 = SData::new2(300, 3);
  let sd_500_5 = SData::new2(500, 5);
  let sd_800_8 = SData::new2(800, 8);
  let sd_1000_10 = SData::new2(1000, 10);
  let sd_1500_15 = SData::new2(1500, 15);

  let mut group = c.benchmark_group("solve");
  group.sample_size(30);
  group.bench_function(
    "solve_e_300_3",
    |b| b.iter_custom(|_| measure(&state1, &sd_300_3))
  );
  group.bench_function(
    "solve_e_500_5",
    |b| b.iter_custom(|_| measure(&state1, &sd_500_5))
  );
  group.bench_function(
    "solve_e_800_8",
    |b| b.iter_custom(|_| measure(&state1, &sd_800_8))
  );
  group.bench_function(
    "solve_h_800_8",
    |b| b.iter_custom(|_| measure(&state2, &sd_800_8))
  );
  group.bench_function(
    "solve_h_1000_10",
    |b| b.iter_custom(|_| measure(&state2, &sd_1000_10))
  );
  group.bench_function(
    "solve_h_1500_15",
    |b| b.iter_custom(|_| measure(&state2, &sd_1500_15))
  );
  group.finish();
}

criterion_group!(benches, solve_bench);
criterion_main!(benches);
