use rand::Rng;
use rayon::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use std::ops::Range;

use crate::ds::*;
use super::cache::Cache;
use super::analysis::HData;
use super::state::{State, SData};

pub struct DataGenerator {
  pub gwb: WBank,
  pub awb: WBank,
  pub wlen: u8,
  pub hd: HData,
  pub cache: Cache, // TODO using the same cache messes with time results
  pub alens: Range<usize>,
  pub turns: Range<u32>,
  pub ntops: Range<u32>,
  pub ecuts: Range<u32>,
  pub ccuts: Range<u32>,
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
      let alen = rng.gen_range(self.alens.clone());
      let turns = rng.gen_range(self.turns.clone());
      let ntops = rng.gen_range(self.ntops.clone());
      let ecut = rng.gen_range(self.ecuts.clone());
      let ccut = rng.gen_range(self.ccuts.clone());
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
