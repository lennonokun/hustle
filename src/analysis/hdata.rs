use std::fs::File;
use std::io::{BufRead, BufReader, Result};
use std::path::Path;

use crate::ds::*;

// loaded approximated heuristics
// does this really need to be f64
#[derive(Debug, Clone)]
pub struct HData {
  approxs: Vec<f64>,
  lbounds: Vec<u32>,
}

impl HData {
  pub fn load<P>(p1: &P) -> Result<Self>
  where P: AsRef<Path> + ?Sized, {
    let p2 = "/home/lokun/code/hustle/data/lbs.csv";
    let reader1 = BufReader::new(File::open(p1)?);
    let reader2 = BufReader::new(File::open(p2)?);
    let approxs = reader1
      .lines()
      .skip(1)
      .filter_map(|s| {
        let s = s.ok()?;
        let s = s.split(",").nth(1)?;
        s.parse::<f64>().ok()
      })
      .collect::<Vec<f64>>();
    let lbounds = reader2
      .lines()
      .skip(1)
      .filter_map(|s| {
        let s = s.ok()?;
        let s = s.split(",").nth(1)?;
        s.parse::<u32>().ok()
      })
      .collect::<Vec<u32>>();

    Ok(Self {approxs, lbounds})
  }

  #[inline]
  pub fn get_approx(&self, n: usize) -> Option<f64> {
    self.approxs.get(n-1).map(|x| *x)
  }

  #[inline]
  pub fn get_lbound(&self, n: usize) -> Option<u32> {
    self.lbounds.get(n-1).map(|x| *x)
  }
}
