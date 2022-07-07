use rand::prelude::*;
use rayon::prelude::*;
use std::sync::Mutex;

use crate::ds::*;
use crate::solve::analysis::HData;

// maximum number of words solveable in two guesses
const MAX_TWOSOLVE: u32 = 20;

#[derive(Clone, Copy)]
pub struct Config {
  // number of top words to try
  pub ntops: u32,
  // number of remaining words makes it "endgame"
  pub endgcutoff: u32,
  // hard mode flag
  pub hard: bool,
}

struct SolveData {
  dt: Option<DTree>,
  beta: u32,
}

#[derive(Clone)]
pub struct State<'a> {
  pub gws: Vec<Word>,
  pub aws: Vec<Word>,
  pub wlen: u32,
  pub n: u32,
  pub cfg: &'a Config,
  pub hd: &'a HData,
}

pub fn fb_filter(gw: Word, fb: Feedback, gws: &Vec<Word>) -> Vec<Word> {
  gws
    .iter()
    .cloned()
    .filter(|gw2| fb == Feedback::from(gw, *gw2).unwrap())
    .collect()
}

impl<'a> State<'a> {
  pub fn new(gws: Vec<Word>, aws: Vec<Word>, wlen: u32,
             cfg: &'a Config, hd: &'a HData) -> Self {
    State {
      gws,
      aws,
      wlen,
      n: wlen + NEXTRA as u32,
      cfg,
      hd
    }
  }

  pub fn new2(gws: Vec<Word>, aws: Vec<Word>, wlen: u32, n: u32,
              cfg: &'a Config, hd: &'a HData) -> Self {
    State { gws, aws, wlen, n, cfg, hd }
  }

  pub fn fb_follow(self, gw: Word, fb: Feedback) -> State<'a> {
    let gws = if self.cfg.hard {fb_filter(gw, fb, &self.gws)} else {self.gws};
    let aws = fb_filter(gw, fb, &self.aws);
    State::new2(gws, aws, self.wlen, self.n-1, self.cfg, self.hd)
  }

  pub fn fb_partition(&self, gw: &Word) -> FbMap<State> {
    let mut map = FbMap::new();
    for aw in &self.aws {
      let fb = Feedback::from(*gw, *aw).unwrap();
      let s2: &mut State = map.entry(fb).or_insert_with(|| {
        let gws2 = if self.cfg.hard {
          fb_filter(*gw, fb, &self.gws)
        } else {
          self.gws.clone()
        };
        State::new2(gws2, Vec::new(), self.wlen, self.n - 1, self.cfg, self.hd)
      });
      s2.aws.push(*aw);
    }
    map
  }

  pub fn fb_counts(&self, gw: &Word) -> FbMap<u32> {
    let mut map = FbMap::new();
    for aw in &self.aws {
      let fb = Feedback::from(*gw, *aw).unwrap();
      *map.entry(fb).or_insert(0) += 1;
    }
    map
  }

  pub fn heuristic(&self, gw: &Word) -> f64 {
    let h = self
      .fb_counts(gw)
      .iter()
      .map(|(_, n)| self.hd.get_approx(*n as usize))
      .sum();
    if self.aws.contains(gw) {
      h - 1.
    } else {
      h
    }
  }

  pub fn top_words(&self) -> Vec<Word> {
    let mut tups: Vec<(Word, f64)> = self
      .gws
      .iter()
      .map(|gw| (*gw, self.heuristic(gw)))
      .collect();
    tups.sort_by(|(_, f1), (_, f2)| f1.partial_cmp(f2).unwrap());
    tups
      .iter()
      .map(|(gw, _)| *gw)
      .take(self.cfg.ntops as usize)
      .collect()
  }

  pub fn solve_given(&self, gw: Word, beta: u32) -> Option<DTree> {
    let alen = self.aws.len();
    if alen == 1 && gw == *self.aws.get(0).unwrap() {
      // leaf if guessed
      return Some(DTree::Leaf);
    } else if self.n == 0 || (self.n == 1 && alen > MAX_TWOSOLVE as usize) {
      // impossible guesses
      return None;
    }

    let mut tot = alen as u32;
    let mut fbm = FbMap::new();

    for (&fb, s2) in self.fb_partition(&gw).iter() {
      if fb.is_correct() {
        fbm.insert(fb, DTree::Leaf);
      } else {
        match s2.solve(beta) {
          None => return None,
          Some(dt) => {
            tot += dt.get_tot();
            fbm.insert(fb, dt);
            if tot >= beta {
              return None;
            }
          }
        }
      }
    }

    Some(DTree::Node {
      tot,
      word: gw,
      fbmap: fbm,
    })
  }

  pub fn solve(&self, beta: u32) -> Option<DTree> {
    let alen = self.aws.len();

    // no more turns
    if self.n == 0 {
      return None;
    // one answer -> guess it
    } else if alen == 1 {
      return Some(DTree::Node {
        tot: 1,
        word: *self.aws.get(0).unwrap(),
        fbmap: [(Feedback::from_str("GGGGG").unwrap(), DTree::Leaf)].into(),
      });
    }

    // check if endgame guess is viable
    if alen <= self.cfg.endgcutoff as usize {
      for aw in self.aws.iter() {
        if self.fb_counts(aw).values().all(|c| *c == 1) {
          return self.solve_given(*aw, beta);
        }
      }
    }

    // finally, check top words
    //    let sd = Mutex::new(SolveData { dt: None, beta });
    //    self.top_words(cfg, hd).into_par_iter().map(|w| {
    //      if sd.lock().unwrap().beta <= 2 * alen as u32 {return}
    //      let dt2 = self.solve_given(w, cfg, hd, sd.lock().unwrap().beta);
    //      let mut sd2 = sd.lock().unwrap();
    //      if let Some(dt2) = dt2 {
    //        if dt2.get_tot() < sd2.beta {
    //          sd2.beta = dt2.get_tot();
    //          sd2.dt = Some(dt2);
    //        }
    //      }
    //    });
    //
    //   sd.into_inner().unwrap().dt
    let mut dt = None;
    let mut beta = beta;
    for w in self.top_words() {
      if beta <= 2 * alen as u32 {
        break;
      }

      let dt2 = self.solve_given(w, beta);
      if let Some(dt2) = dt2 {
        if dt2.get_tot() < beta {
          beta = dt2.get_tot();
          dt = Some(dt2);
        }
      }
    }
    dt
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn simple_solve() {
    let wlen = 5;
    let mut cfg = Config {ntops: 2, endgcutoff: 15, hard: false};
    let hd = HData::load("/usr/share/hustle/happrox.csv").unwrap();
    let (gwb, awb) = WBank::from2("/usr/share/hustle/bank1.csv", wlen).unwrap();
    let state = State::new(gwb.data, awb.data, wlen.into(), &cfg, &hd);
    assert!(state.solve(u32::MAX).is_some());
    let mut cfg = Config {ntops: 2, endgcutoff: 15, hard: true};
    let hd = HData::load("/usr/share/hustle/happrox.csv").unwrap();
    let (gwb, awb) = WBank::from2("/usr/share/hustle/bank1.csv", wlen).unwrap();
    let state = State::new(gwb.data, awb.data, wlen.into(), &cfg, &hd);
    assert!(state.solve(u32::MAX).is_some());
  }
}

