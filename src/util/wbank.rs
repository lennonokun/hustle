use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::fmt;

use rand::prelude::*;

use super::word::Word;
use super::misc::*;

#[derive(Debug, Clone)]
pub struct WBank {
  pub gws: Vec<Word>,
  pub aws: Vec<Word>,
  pub wlen: u8,
}

impl WBank {
  pub fn load<P: AsRef<Path>>(p: &P, wlen: u8) -> io::Result<Self> {
    let file = File::open(p)?;
    let reader = BufReader::new(file);
    let mut gws = Vec::<Word>::new();
    let mut aws = Vec::<Word>::new();
    for line in reader.lines().skip(1).flatten() {
      // parse line
      let vec: Vec<&str> = line.split(',').collect();
      if vec[2].parse::<u8>().unwrap() != wlen {
        continue;
      }
      // push to both if answer word, but only guess if guess word
      let w = Word::from_str(vec[0]).unwrap();
      if vec[1] == "A" {
        aws.push(w)
      }
      gws.push(w);
    }

    Ok(Self {gws, aws, wlen})
  }

  pub fn load1() -> io::Result<Self> {
    Self::load(&DEFWBP, DEFWLEN)
  }

  pub fn load2(wlen: u8) -> io::Result<Self> {
    Self::load(&DEFWBP2, wlen)
  }

  pub fn glen(&self) -> usize {
    self.gws.len()
  }

  pub fn alen(&self) -> usize {
    self.aws.len()
  }

  pub fn contains_gw(&self, gw: Word) -> bool {
    self.gws.contains(&gw)
  }

  pub fn contains_aw(&self, aw: Word) -> bool {
    self.aws.contains(&aw)
  }

  pub fn choose_gw(&self, rng: &mut ThreadRng) -> Word {
    *self.gws.choose(rng).unwrap()
  }

  pub fn choose_aw(&self, rng: &mut ThreadRng) -> Word {
    *self.aws.choose(rng).unwrap()
  }

  pub fn sample_gws(&self, rng: &mut ThreadRng, n: usize) -> Vec<Word> {
    self.gws.choose_multiple(rng, n).cloned().collect()
  }

  pub fn sample_aws(&self, rng: &mut ThreadRng, n: usize) -> Vec<Word> {
    self.aws.choose_multiple(rng, n).cloned().collect()
  }
  
  pub fn sample(
    &self,
    rng: &mut ThreadRng,
    glen: Option<usize>,
    alen: Option<usize>
  ) -> WBank {
    let gws = self.sample_gws(rng, glen.unwrap_or(self.glen()));
    let aws = self.sample_aws(rng, alen.unwrap_or(self.alen()));
    WBank {gws, aws, wlen: self.wlen}
  }
}
