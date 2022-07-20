use rand::Rng;
use rand::rngs::ThreadRng;
use rand::distributions::Distribution;
use rand::distributions::uniform::{Uniform, SampleUniform};
use rayon::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use std::cmp::PartialOrd;
use std::str::FromStr;
use std::ops::Add;

use crate::ds::*;
use super::cache::Cache;
use super::analysis::HData;
use super::state::{State, SData};

pub fn parse_uniform<X>(s: &String) -> Option<Uniform<X>> 
where X: SampleUniform + FromStr { 
  lazy_static! {
    static ref RE_UNIF: Regex = Regex::new(r"^(\d+)..(=?)(\d+)$").unwrap();
  }
  if let Some(caps) = RE_UNIF.captures(s) {
    let a: X = caps.get(1)?.as_str().parse().ok()?;
    let b: X = caps.get(3)?.as_str().parse().ok()?;
    if caps.get(2)?.as_str().is_empty() {
      Some(Uniform::new(a, b))
    } else {
      Some(Uniform::new_inclusive(a, b))
    }
  } else {
    None
  }
}

pub struct DataGenerator {
  pub gwb: WBank,
  pub awb: WBank,
  pub wlen: u8,
  pub hd: HData,
  pub cache: Cache, // TODO using the same cache interferes with time results
  pub alens: Uniform<usize>,
  pub turns: Uniform<u32>,
  pub ntops: Uniform<u32>,
  pub ecuts: Uniform<u32>,
  pub ccuts: Uniform<u32>,
  pub niter: usize,
}

impl DataGenerator {
  pub fn run(&mut self, out: &Path) {
    // open and write header if new
    let mut f;
    if out.exists() {
      f = OpenOptions::new()
        .write(true)
        .append(true)
        .open(out)
        .unwrap();
    } else {
      f = File::create(out).unwrap();
      writeln!(&mut f, "alen,tot,time,turns,mode,ntops,ecut,ccut");
    }

    // generate data
    let f = Mutex::new(f);
    let i = Mutex::new(1);
    (0..self.niter).into_par_iter().for_each(|_| {
      // take samples
      let mut rng = rand::thread_rng();
      let alen = self.alens.sample(&mut rng);
      let turns = self.turns.sample(&mut rng);
      let ntops = self.ntops.sample(&mut rng);
      let ecut = self.ecuts.sample(&mut rng);
      let ccut = self.ccuts.sample(&mut rng);
      let hard = false; // FOR NOW ALWAYS EASY BC CACHE DONT CHECK GWS

      // make state
      let aws2 = self.awb.pick(&mut rng, alen as usize);
      let s = State::new2(self.gwb.data.clone(), aws2, self.awb.wlen.into(), turns, false);
      let mut sd = SData::new(self.hd.clone(), self.cache.clone(), ntops, ecut, ccut);

      // solve and time
      let instant = Instant::now();
      let dt = s.solve(&mut sd, u32::MAX);
      let time = instant.elapsed().as_millis();
      let tot = dt.map_or(u32::MAX, |dt| dt.get_tot());

      // print and write results to file
      let mut i = i.lock().unwrap();
      let mut f = f.lock().unwrap();
      let s = format!(
        "{},{},{},{},{},{},{},{}",
        alen,
        tot,
        time,
        turns,
        if hard { "H" } else { "E" },
        ntops,
        ecut,
        ccut,
      );
      println!("{}. {}", *i, s);
      writeln!(f, "{}", s);
      *i += 1;
    });
  }
}
