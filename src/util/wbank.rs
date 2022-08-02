use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use rand::prelude::*;

use super::word::Word;

#[derive(Debug, Clone)]
pub struct WBank {
  pub data: Vec<Word>,
  pub wlen: u8,
}

impl WBank {
  pub fn from2<P>(p: P, wlen: u8) -> io::Result<(Self, Self)>
  where
    P: AsRef<Path>, {
    let file = File::open(p)?;
    let reader = BufReader::new(file);
    let mut gdata = Vec::<Word>::new();
    let mut adata = Vec::<Word>::new();
    for line in reader.lines().skip(1).flatten() {
      // parse line
      let vec: Vec<&str> = line.split(',').collect();
      if vec[2].parse::<u8>().unwrap() != wlen {
        continue;
      }
      // push to both if answer word, but only guess if guess word
      let w = Word::from_str(vec[0]).unwrap();
      if vec[1] == "A" {
        adata.push(w)
      }
      gdata.push(w);
    }

    Ok((WBank { data: gdata, wlen }, WBank { data: adata, wlen }))
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn new() -> Self {
    WBank {
      data: Vec::new(),
      wlen: 0,
    }
  }

  pub fn new2(wlen: u8) -> Self {
    WBank {
      data: Vec::new(),
      wlen,
    }
  }

  pub fn contains(&self, w: Word) -> bool {
    self.data.contains(&w)
  }

  pub fn pick(&self, rng: &mut ThreadRng, n: usize) -> Vec<Word> {
    self.data.choose_multiple(rng, n).cloned().collect()
  }

  pub fn to_string(&self) -> String {
    let s = self
      .data
      .iter()
      .map(|w| w.to_string())
      .collect::<Vec<String>>()
      .join(" ");
    format!("[{s}]")
  }
}
