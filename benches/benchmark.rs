use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkGroup};
use criterion::measurement::Measurement;
use hustle::solve::{State, SData, MState, MData, AData, Cache};
use hustle::util::*;
use std::time::{Instant, Duration};

pub fn single_solve_bench(c: &mut Criterion) {
  let gw = Word::from_str("SALET").unwrap();
  let state_e = State::new3();
  let mut state_h = State::new3();
  state_h.hard = true;

  let mut group = c.benchmark_group("solve");
  group.sample_size(30);

  // bench easy
  for (nwords1, nwords2) in vec![(300,3), (500,5), (800,8)] {
    let name = format!("single_solve_e_{nwords1}_{nwords2}");
    let sdata = SData::new2(nwords1, nwords2);
    group.bench_function(name, |b| b.iter(|| {
      black_box(&state_e).solve_given(gw, &sdata, u32::MAX);
    }));
  }

  // bench hard
  for (nwords1, nwords2) in vec![(800,8), (1000,10), (1500,15)] {
    let name = format!("single_solve_h_{nwords1}_{nwords2}");
    let sdata = SData::new2(nwords1, nwords2);
    group.bench_function(name, |b| b.iter(|| {
      black_box(&state_h).solve_given(gw, &sdata, u32::MAX);
    }));
  }
  group.finish();
}

pub fn multi_solve_bench(c: &mut Criterion) {
  let (gwb, awb) = WBank::from2("/usr/share/hustle/bank1.csv", 5).unwrap();
  let (gws, aws) = (gwb.data, awb.data);
  let gw = Word::from_str("SALET").unwrap();

  let mut group = c.benchmark_group("multi_solve");
  group.sample_size(10);
  for nwords in vec![2, 4, 6] {
    for (nguesses, nanswers) in vec![(3,3), (5,5)] {
      let name = format!("multi_solve_{nwords}_{nguesses}_{nanswers}");
      let state = MState::new(gws.clone(), vec![aws.clone(); nwords as usize], 5, nwords, false);
      let mut mdata = MData::new2(nguesses, nanswers);
      
      group.bench_function(name, |b| b.iter(|| {
        black_box(&state).solve_given(gw, &mut mdata);
      }));
    }
  }
  group.finish();
}


criterion_group!(benches, single_solve_bench, multi_solve_bench);
criterion_main!(benches);
