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
use crate::solve::{State, SData, HData, Cache};

// TODO default settings to out's settings if existed

pub struct GGen {
  pub gwb: WBank,
  pub awb: WBank,
  pub wlen: u32,
  pub hd: HData,
  pub cache: Cache, // TODO using the same cache interferes with time results
  pub alens: Range<usize>,
  pub turns: Range<u32>,
  pub ntops1: Range<u32>,
  pub ntops2: Range<u32>,
  pub ecuts: Range<u32>,
  pub niter: usize,
}

impl GGen { 
  fn header() -> &'static str {
    "alen,tot,time,turns,mode,ntops1,ntops2,ecut"
  }

  fn metadata(&self) -> Vec<String> {
    vec![
      "# kind: sgen".to_owned(),
      format!("# alens: {}", self.alens),
      format!("# turns: {}", self.turns),
      format!("# ntops1: {}", self.ntops1),
      format!("# ntops2: {}", self.ntops2),
      format!("# ecuts: {}", self.ecuts),
    ]
  }

  // open, check formatting, append if existing
  fn open_file(&self, out: &Path) -> Result<File, Error> {
    let existed = out.exists();
    let meta = self.metadata();
    let mut f = OpenOptions::new()
      .create(true)
      .append(true)
      .open(out)?;

    if existed {
      // check metadata
      let f = File::open(out)?;
      let reader = BufReader::new(f);
      let mut lines = reader.lines();

      // check first lines of metadata
      for meta_line in &meta {
        if &lines.next().ok_or(Error::new(ErrorKind::Other,"not enough lines!"))?? != meta_line {
          return Err(Error::new(
            ErrorKind::Other,
            "metadata does not match!"
          ));
        }
      }
    } else {
      // write metadata + header if new
      writeln!(f, "{}", meta.join("\n"));
      writeln!(f, "{}", Self::header());
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
      let ntops1 = self.ntops1.sample(&mut rng);
      let ntops2 = self.ntops2.sample(&mut rng);
      let ecut = self.ecuts.sample(&mut rng);
      let hard = false; // FOR NOW ALWAYS EASY BC CACHE DOESNT CHECK GWS

      // make state
      let aws2 = self.awb.pick(&mut rng, alen as usize);
      let s = State::new2(self.gwb.data.clone(), aws2, self.wlen, turns as u32, false);
      let mut sd = SData::new(self.hd.clone(), self.cache.clone(),
                              ntops1 as u32, ntops2 as u32, ecut as u32);

      // solve and time
      let instant = Instant::now();
      let dt = s.solve(&sd, u32::MAX);
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
        ntops1,
        ntops2,
        ecut,
      );
      println!("{}. {}", *i, s);
      writeln!(f, "{}", s);
      *i += 1;
    });

    Ok(())
  }
}
