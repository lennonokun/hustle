use std::fs::File;
use std::io::{BufRead, BufReader, Result, Write};
use std::path::Path;
use std::sync::Mutex;

use crate::ds::*;

// loaded heuristic data
// let ht[0] be zero to make indexing easier, so +1
// does this really need to be f64
pub struct HData {
  approx: [f64; NWORDS + 1],
}

impl HData {
  pub fn load<P>(p: P) -> Result<Self>
  where
    P: AsRef<Path>, {
    let file = File::open(p)?;
    let reader = BufReader::new(file);
    let approx: [f64; NWORDS + 1] = reader
      .lines()
      .filter_map(|s| s.ok()?.parse::<f64>().ok())
      .collect::<Vec<f64>>()
      .try_into()
      .expect("expected NWORDS+1 lines in heuristic cache");
    Ok(Self { approx })
  }

  #[inline]
  pub fn get_approx(&self, n: usize) -> f64 {
    self.approx[n]
  }
}
