use std::time::Duration;
use std::collections::HashMap;

use rand::prelude::*;
use rand::seq::SliceRandom;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use hustle::solve::*;
use hustle::util::*;

pub fn fbmap_bench(c: &mut Criterion) {
  let mut rng = thread_rng();

  let mut group = c.benchmark_group("fbmap");
  group.warm_up_time(Duration::from_secs(1));
  for wlen in 4..=6 {
    let (gwb, awb) = WBank::from2("/usr/share/hustle/bank2.csv", wlen).unwrap();
    for alen in vec![15] {
      let gw = *gwb.data.choose(&mut rng).unwrap();
      let aws = awb.pick(&mut rng, alen);

      group.bench_function(format!("auto_{wlen}_{alen}_u16"), |b| b.iter(|| {
        let mut autofbmap = AutoFbMap::<u16>::new(wlen, alen, 0);
        for aw in aws.iter().copied() {
          black_box(*autofbmap.get_mut(gw, aw));
        }
        for (fb, n) in autofbmap.into_iter() {
          black_box((fb, n));
        }
      }));
      group.bench_function(format!("vec_{wlen}_{alen}_u16"), |b| b.iter(|| {
        let mut vec = vec![0u16; 3usize.pow(wlen as u32)];
        for aw in aws.iter().copied() {
          black_box(vec[fb_id(gw, aw) as usize]);
        }
        for (id, n) in vec.iter_mut().enumerate() {
          black_box((Feedback::from_id(id as u32, wlen), n));
        }
      }));
      group.bench_function(format!("map_{wlen}_{alen}_u16"), |b| b.iter(|| {
        let mut map = HashMap::<Feedback, u16>::new();
        for aw in aws.iter().copied() {
          black_box(map.entry(Feedback::from(gw, aw).unwrap()).or_insert(0));
        }
        for (fb, n) in map.iter() {
          black_box((fb, n));
        }
      }));
    }
  }
  group.finish();
}

pub fn top_words_bench(c: &mut Criterion) {
  let wlen = 5;
  let (gwb, awb) = WBank::from2("/usr/share/hustle/bank2.csv", wlen).unwrap();
  let mut rng = rand::thread_rng();

  let mut group = c.benchmark_group("top_words");
  group.warm_up_time(Duration::from_secs(1));
  group.sample_size(30);

  for glen in vec![1000, 10000] {
    for alen in vec![10, 100, 1000, 10000] {
      let gws = gwb.data.choose_multiple(&mut rng, glen).cloned().collect();
      let aws = awb.data.choose_multiple(&mut rng, alen).cloned().collect();
      let state = State::new(gws, aws, wlen.into(), false);
      let sd = SData::new2(1000, 10);
      group.bench_function(format!("top_words_{glen}_{alen}"), |b| b.iter(|| {
        black_box(&state).top_words(&sd);
      }));
    }
  }

  group.finish();
}

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


criterion_group!(
  benches,
  // fbmap_bench,
  top_words_bench,
  single_solve_bench,
  multi_solve_bench
);
criterion_main!(benches);
