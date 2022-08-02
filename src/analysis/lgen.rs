use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Write, BufRead, BufReader};
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;
use std::collections::HashMap;
use std::sync::Arc;

use rand::Rng;
use rand::rngs::ThreadRng;
use rand::distributions::{Distribution, Uniform};
use rayon::prelude::*;

use crate::util::*;
use crate::solve::{State, SData, AData, Cache};

// TODO default settings to out's settings if existed

pub struct LGen {
  pub gwb: WBank,
  pub awb: WBank,
  pub wlen: u32,
  pub adata: AData,
  pub cache: Cache, // TODO using the same cache interferes with time results
  pub turns: u32,
  pub alens: Range<usize>,
  pub ntops1: u32,
  pub ntops2: u32,
  pub ecut: u32,
  pub niter: usize,
  pub step: usize,
}

impl LGen {
  fn header() -> &'static str {
    "alen,lb"
  }

  fn meta(&self) -> Vec<String> {
    vec![
      "# kind: lgen".to_owned(),
      format!("# alens: {}", self.alens),
      format!("# step: {}", self.step),
      format!("# turns: {}", self.turns),
      format!("# ntops1: {}", self.ntops1),
      format!("# ntops2: {}", self.ntops2),
      format!("# ecut: {}", self.ecut),
      format!("# step: {}", self.step),
    ]
  }

  // open, check formatting, get previous bounds
  pub fn open_file(&self, out: &Path) -> Result<(File, HashMap<usize, u32>), Error> {
    let existed = out.exists();
    let meta = self.meta();
    let mut f = OpenOptions::new()
      .write(true)
      .create(true)
      .open(out)?;

    // check metadata and get previous lbs
    let lbs = if existed {
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

      // get previous lower bounds
      lines.filter_map(|s| {
        let s = s.ok()?;
        let mut split = s.split(",");
        let alen = split.next()?.parse::<usize>().ok()?;
        let lb = split.next()?.parse::<u32>().ok()?;
        Some((alen, lb))
      }).collect::<HashMap<usize, u32>>()
    } else {
      HashMap::new()
    };

    // write metadata + header
    writeln!(f, "{}", meta.join("\n"));
    writeln!(f, "{}", Self::header());

    Ok((f, lbs))
  }

  pub fn run(&mut self, out: &Path) -> Result<(), Error> {
    // generate data in parallel
    let (f, lbs) = self.open_file(out)?;
    let f = Mutex::new(f);
    let lbs = Mutex::new(lbs);
    let i = Mutex::new(1);

    let alens: Vec<usize> = (self.alens.a..=self.alens.b).step_by(self.step).collect();
    alens.into_par_iter().for_each(|alen| {
      let mut lb = lbs.lock().unwrap().get(&alen).map(|x| *x).unwrap_or(u32::MAX);
      let mut rng = rand::thread_rng();

      for _ in 0..self.niter {
        // make state
        let aws2 = self.awb.pick(&mut rng, alen as usize);
        let s = State::new2(Arc::new(self.gwb.data.clone()), aws2, self.wlen, self.turns as u32, false);
        let mut sd = SData::new(self.adata.clone(), self.cache.clone(),
                                self.ntops1 as u32, self.ntops2, self.ecut as u32);

        // solve and time
        let dt = s.solve(&sd, u32::MAX);
        let tot = dt.map_or(u32::MAX, |dt| dt.get_tot());
        if tot < lb {
          lb = tot;
        }
      }

      // print and write results to file
      let mut i = i.lock().unwrap();
      let mut f = f.lock().unwrap();
      let s = format!(
        "{},{}",
        alen,
        lb,
      );
      println!("{}. {}", *i, s);
      writeln!(f, "{}", s);
      *i += 1;
    });

    Ok(())
  }
}
