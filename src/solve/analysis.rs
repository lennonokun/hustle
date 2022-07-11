use std::fs::File;
use std::io::{BufRead, BufReader, Result, Write};
use std::path::Path;
use std::sync::Mutex;

use crate::ds::*;

// loaded approximated heuristics
// does this really need to be f64
#[derive(Debug,Clone)]
pub struct HData {
  approx: [f64; NWORDS],
}

impl HData {
  pub fn load<P>(p: &P) -> Result<Self>
  where
    P: AsRef<Path>+?Sized, {
    let file = File::open(p)?;
    let reader = BufReader::new(file);
    let approx = reader.lines().skip(1)
      .filter_map(|s| {
        let mut s = s.ok()?;
        let mut s = s.split(",").nth(1)?;
        s.parse::<f64>().ok()
      }).collect::<Vec<f64>>();
    Ok(Self { approx: approx.try_into().unwrap() })
  }

  #[inline]
  pub fn get_approx(&self, n: usize) -> f64 {
    self.approx[n-1]
  }
}

