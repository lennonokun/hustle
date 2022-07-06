use rand::prelude::*;
use rayon::prelude::*;
use std::sync::Mutex;

use crate::ds::*;

pub mod analysis;
use crate::solve::analysis::HData;
pub mod util;
use crate::solve::util::*;

#[derive(Clone, Copy)]
pub struct Config {
  // number of top words to try
  pub ntops: u32,
  // number of remaining words makes it "endgame"
  pub endgcutoff: u32,
  // hard mode flag
  pub hard: bool,
}

#[derive(Clone)]
pub struct State {
  pub gws: Vec<Word>,
  pub aws: Vec<Word>,
  pub wlen: u32,
  pub n: u32,
}

struct SolveData {
  dt: Option<DTree>,
  beta: u32,
}

pub fn fb_filter(gw: Word, fb: Feedback, gws: &Vec<Word>) -> Vec<Word> {
  gws
    .iter()
    .cloned()
    .filter(|gw2| fb == Feedback::from(gw, *gw2).unwrap())
    .collect()
}

impl State {
  pub fn new(gws: Vec<Word>, aws: Vec<Word>, wlen: u32) -> Self {
    State {
      gws,
      aws,
      wlen,
      n: wlen + NEXTRA as u32,
    }
  }

  pub fn new2(gws: Vec<Word>, aws: Vec<Word>, wlen: u32, n: u32) -> Self {
    State { gws, aws, wlen, n }
  }

  pub fn fb_partition(&self, gw: &Word, cfg: &Config) -> FbMap<State> {
    let mut map = FbMap::new();
    for aw in &self.aws {
      let fb = Feedback::from(*gw, *aw).unwrap();
      let s2: &mut State = map.entry(fb).or_insert_with(|| {
        let gws2 = if cfg.hard {
          fb_filter(*gw, fb, &self.gws)
        } else {
          self.gws.clone()
        };
        State::new2(gws2, Vec::new(), self.wlen, self.n - 1)
      });
      s2.aws.push(*aw);
    }
    map
  }

  pub fn fb_counts(&self, gw: &Word, cfg: &Config) -> FbMap<u32> {
    let mut map = FbMap::new();
    for aw in &self.aws {
      let fb = Feedback::from(*gw, *aw).unwrap();
      *map.entry(fb).or_insert(0) += 1;
    }
    map
  }

  pub fn heuristic(&self, gw: &Word, cfg: &Config, hd: &HData) -> f64 {
    let h = self
      .fb_counts(gw, cfg)
      .iter()
      .map(|(_, n)| hd.get_approx(*n as usize))
      .sum();
    if self.aws.contains(gw) {
      h - 1.
    } else {
      h
    }
  }

  pub fn top_words(&self, cfg: &Config, hd: &HData) -> Vec<Word> {
    let mut tups: Vec<(Word, f64)> = self
      .gws
      .iter()
      .map(|gw| (*gw, self.heuristic(gw, cfg, hd)))
      .collect();
    tups.sort_by(|(_, f1), (_, f2)| f1.partial_cmp(f2).unwrap());
    tups
      .iter()
      .map(|(gw, _)| *gw)
      .take(cfg.ntops as usize)
      .collect()
  }

  pub fn solve_given(&self, gw: Word, cfg: &Config, hd: &HData, beta: u32) -> Option<DTree> {
    let alen = self.aws.len();
    if alen == 1 && gw == *self.aws.get(0).unwrap() {
      // leaf if guessed
      return Some(DTree::Leaf);
    } else if self.n == 0 || (self.n == 1 && alen > 20) {
      // impossible guesses
      return None;
    }

    let mut tot = alen as u32;
    let mut fbm = FbMap::new();

    for (&fb, s2) in self.fb_partition(&gw, &cfg).iter() {
      if fb.is_correct() {
        fbm.insert(fb, DTree::Leaf);
      } else {
        match s2.solve(cfg, hd, beta) {
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

  pub fn solve(&self, cfg: &Config, hd: &HData, beta: u32) -> Option<DTree> {
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
    if alen <= cfg.endgcutoff as usize {
      for aw in self.aws.iter() {
        if self.fb_counts(aw, cfg).values().all(|c| *c == 1) {
          return self.solve_given(*aw, cfg, hd, beta);
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
    for w in self.top_words(cfg, hd) {
      if beta <= 2 * alen as u32 {
        break;
      }

      let dt2 = self.solve_given(w, cfg, hd, beta);
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
