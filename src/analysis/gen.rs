use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Write, BufRead, BufReader};
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

use rand::Rng;
use rand::rngs::ThreadRng;
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;

use crate::ds::*;
use crate::solve::{State, SData, Cache};

use super::hdata::HData;
use super::range::Range;

// TODO default settings to out's settings if existed

const LEN_METADATA: usize = 4;

pub struct SGen {
  pub gwb: WBank,
  pub awb: WBank,
  pub wlen: u32,
  pub hd: HData,
  pub cache: Cache, // TODO using the same cache interferes with time results
  pub alens: Range<usize>,
  pub turns: Range<u32>,
  pub ntops: Range<u32>,
  pub ecuts: Range<u32>,
  pub niter: usize,
}

impl SGen { 
  pub fn metadata(&self) -> String {
    let mut s = String::new();
    s += &format!("# alens: {}\n", self.alens);
    s += &format!("# turns: {}\n", self.turns);
    s += &format!("# ntops: {}\n", self.ntops);
    s += &format!("# ecuts: {}\n", self.ecuts);
    s
  }

  // open, check formatting, append if existing
  pub fn open_file(&self, out: &Path) -> Result<File, Error> {
    // create and check if already existed
    let existed = out.exists();
    let metadata = self.metadata();
    let mut f = OpenOptions::new()
      .create(true)
      .append(true)
      .open(out)?;

    // check metadata if existed
    if existed {
      let f = File::open(out)?;
      let reader = BufReader::new(f);
      let existing_metadata = reader
        .lines()
        .take(LEN_METADATA)
        .filter_map(|x| x.ok())
        .collect::<Vec<String>>().join("\n") + "\n";
      if existing_metadata != metadata {
        return Err(Error::new(
          ErrorKind::Other,
          "metadata does not match!"
        ));
      }
    }

    let mut f = OpenOptions::new()
      .create(true)
      .append(true)
      .open(out)?;

    // write metadata + header if new
    if !existed {
      write!(f, "{}", metadata);
      write!(f, "alen,tot,time,turns,mode,ntops,ecut\n");
    }

    Ok(f)
  }

  pub fn run(&mut self, out: &Path) -> Result<(), Error> {
    // generate data in parallel
    let f = Mutex::new(self.open_file(out)?);
    let i = Mutex::new(1);
    (0..self.niter).into_par_iter().for_each(|_| {
      // take samples
      let mut rng = rand::thread_rng();
      let alen = self.alens.sample(&mut rng);
      let turns = self.turns.sample(&mut rng);
      let ntops = self.ntops.sample(&mut rng);
      let ecut = self.ecuts.sample(&mut rng);
      let hard = false; // FOR NOW ALWAYS EASY BC CACHE DOESNT CHECK GWS

      // make state
      let aws2 = self.awb.pick(&mut rng, alen as usize);
      let s = State::new2(self.gwb.data.clone(), aws2, self.wlen, turns as u32, false);
      let mut sd = SData::new(self.hd.clone(), self.cache.clone(), ntops as u32, ecut as u32);

      // solve and time
      let instant = Instant::now();
      let dt = s.solve(&mut sd, u32::MAX);
      let time = instant.elapsed().as_millis();
      let tot = dt.map_or(u32::MAX, |dt| dt.get_tot());

      // print and write results to file
      let mut i = i.lock().unwrap();
      let mut f = f.lock().unwrap();
      let s = format!(
        "{},{},{},{},{},{},{}",
        alen,
        tot,
        time,
        turns,
        if hard { "H" } else { "E" },
        ntops,
        ecut,
      );
      println!("{}. {}", *i, s);
      writeln!(f, "{}", s);
      *i += 1;
    });

    Ok(())
  }
}
