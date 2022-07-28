use rand::Rng;
use std::hash::{Hash, Hasher};
use rayon::prelude::*;

use super::cache::Cache;
use crate::analysis::HData;
use crate::ds::*;

// TODO: also hash gws?
// could also iteratively hash when forming the state

// maximum number of words solveable in two guesses
const MAX_TWOSOLVE: u32 = 20;

/// solve data
#[derive(Debug, Clone)]
pub struct SData {
  /// heuristic data
  pub hd: HData,
  /// cache
  pub cache: Cache,
  /// number of top words to try
  pub ntops: u32,
  /// number of remaining words makes it "endgame"
  pub endgcutoff: u32,
}

impl SData {
  pub fn new(hd: HData, cache: Cache, ntops: u32, endgcutoff: u32) -> Self {
    Self {
      hd,
      cache,
      ntops,
      endgcutoff,
    }
  }

  pub fn new2(ntops: u32) -> Self {
    let hd = HData::load(DEFHDP).unwrap();
    let cache = Cache::new(64, 8);
    Self::new(hd, cache, ntops, 15)
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct State {
  pub gws: Vec<Word>,
  pub aws: Vec<Word>,
  pub wlen: u32,
  pub n: u32,
  pub hard: bool,
}

pub fn fb_filter(gw: Word, fb: Feedback, gws: &Vec<Word>) -> Vec<Word> {
  gws
    .iter()
    .cloned()
    .filter(|gw2| fb == Feedback::from(gw, *gw2).unwrap())
    .collect()
}

impl State {
  pub fn new(gws: Vec<Word>, aws: Vec<Word>, wlen: u32, hard: bool) -> Self {
    State {
      gws,
      aws,
      wlen,
      n: NGUESSES as u32,
      hard,
    }
  }

  pub fn new2(gws: Vec<Word>, aws: Vec<Word>, wlen: u32, n: u32, hard: bool) -> Self {
    State {
      gws,
      aws,
      wlen,
      n,
      hard,
    }
  }

  pub fn new3() -> Self {
    let (gwb, awb) = WBank::from2("/usr/share/hustle/bank1.csv", NLETS as u8).unwrap();
    State::new(gwb.data, awb.data, NLETS as u32, false)
  }

  pub fn random(maxlen: usize) -> Self {
    let (gwb, awb) = WBank::from2("/usr/share/hustle/bank1.csv", 5).unwrap();
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(1..=maxlen);
    State::new2(
      gwb.data,
      awb.pick(&mut rng, len),
      NLETS as u32,
      NGUESSES as u32,
      false,
    )
  }

  pub fn fb_follow(self, gw: Word, fb: Feedback) -> Self {
    let gws = if self.hard {
      fb_filter(gw, fb, &self.gws)
    } else {
      self.gws
    };
    let aws = fb_filter(gw, fb, &self.aws);
    State::new2(gws, aws, self.wlen, self.n - 1, self.hard)
  }

  pub fn fb_partition(&self, gw: &Word) -> FbMap<State> {
    let mut map = FbMap::new();
    for aw in &self.aws {
      let fb = Feedback::from(*gw, *aw).unwrap();
      let s2: &mut State = map.entry(fb).or_insert_with(|| {
        let gws2 = if self.hard {
          fb_filter(*gw, fb, &self.gws)
        } else {
          self.gws.clone()
        };
        State::new2(gws2, Vec::new(), self.wlen, self.n - 1, self.hard)
      });
      s2.aws.push(*aw);
    }
    map
  }

  pub fn fb_partition_vec(&self, gw: &Word) -> Vec<(Feedback, State)> {
    let mut awss = vec![Vec::new(); 3usize.pow(self.wlen)];

    for aw in &self.aws {
      awss[fb_id(*gw, *aw) as usize].push(*aw);
    }

    awss.iter()
      .enumerate()
      .filter(|(id, aws)| !aws.is_empty())
      .map(|(id, aws)| {
        let fb = Feedback::from_id(id as u32, self.wlen as u8);
        let gws2 = if self.hard {
          fb_filter(*gw, fb, &self.gws)
        } else {
          self.gws.clone()
        };
        let state = State::new2(gws2, aws.clone(), self.wlen, self.n-1, self.hard);
        (fb, state)
      })
      .collect()
  }

  pub fn fb_counts(&self, gw: &Word) -> FbMap<u32> {
    let mut map = FbMap::new();
    for aw in &self.aws {
      let fb = Feedback::from(*gw, *aw).unwrap();
      *map.entry(fb).or_insert(0) += 1;
    }
    map
  }

  pub fn fb_counts_vec(&self, gw: &Word) -> Vec<u32> {
    // initialize vec with zeros
    let mut vec = vec![0; 3usize.pow(self.wlen)];

    for aw in &self.aws {
      vec[fb_id(*gw, *aw) as usize] += 1;
    }

    vec
  }

  pub fn heuristic(&self, gw: &Word, sd: &SData) -> f64 {
    let h = if self.wlen <= 5 {
      self.fb_counts_vec(gw)
        .iter()
        .filter(|x| **x > 0)
        .map(|x| sd.hd.get_approx(*x as usize).unwrap())
        .sum()
    } else {
      self.fb_counts(gw)
        .iter()
        .map(|(_, n)| sd.hd.get_approx(*n as usize).unwrap())
        .sum()
    };

    if self.aws.contains(gw) {
      h - 1.
    } else {
      h
    }
  }

  pub fn top_words(&self, sd: &SData) -> Vec<Word> {
    let mut tups: Vec<(Word, f64)> = self
      .gws.clone()
      .into_par_iter()
      .map(|gw| (gw, self.heuristic(&gw, sd)))
      .collect();
    tups.sort_by(|(_, f1), (_, f2)| f1.partial_cmp(f2).unwrap());
    tups
      .iter()
      .map(|(gw, _)| *gw)
      .take(sd.ntops as usize)
      .collect()
  }

  pub fn solve_given(&self, gw: Word, sd: &mut SData, beta: u32) -> Option<DTree> {
    let alen = self.aws.len();

    // leaf if guessed
    if alen == 1 && gw == *self.aws.get(0).unwrap() {
      return Some(DTree::Leaf);
    }
    // impossible guesses
    if self.n == 0
      || (self.n == 1 && alen > 1)
      || (self.n == 2 && alen > MAX_TWOSOLVE as usize) {
      return None;
    }
    // check alpha = 2|A|-1
    if beta <= 2 * (alen as u32) - 1 {
      return None;
    }

    let mut tot = alen as u32;
    let mut fbm = FbMap::new();

    if self.wlen <= 5 {
      let fbpv = self.fb_partition_vec(&gw);
      for (fb, s2) in fbpv {
        if fb.is_correct() {
          fbm.insert(fb, DTree::Leaf);
        } else {
          match s2.solve(sd, beta - tot) {
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
    } else {
      let fbp = self.fb_partition(&gw);
      // let mut sfbp: Vec<(&Feedback, &State)> = fbp.iter().collect();
      // sfbp.sort_by_key(|(fb, s)| s.aws.len());

      for (fb, s2) in fbp {
        if fb.is_correct() {
          fbm.insert(fb, DTree::Leaf);
        } else {
          match s2.solve(sd, beta - tot) {
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
    }

    Some(DTree::Node {
      tot,
      word: gw,
      fbmap: fbm,
    })
  }

  pub fn solve(&self, sd: &mut SData, beta: u32) -> Option<DTree> {
    let alen = self.aws.len();

    // no more turns
    if self.n == 0 {
      return None;
    }
    // one answer -> guess it
    if alen == 1 {
      return Some(DTree::Node {
        tot: 1,
        word: *self.aws.get(0).unwrap(),
        fbmap: [(Feedback::from_str("GGGGG").unwrap(), DTree::Leaf)].into(),
      });
    }
    // check alpha = 2|A|-1
    if beta <= 2 * (alen as u32) - 1 {
      return None;
    }
    // check endgame if viable
    if alen <= sd.endgcutoff as usize {
      for aw in self.aws.iter() {
        if self.fb_counts(aw).values().all(|c| *c == 1) {
          return self.solve_given(*aw, sd, beta);
        }
      }
    }
    // check cache
    if !self.hard {
      if let Some(dt) = sd.cache.read(self) {
        return Some(dt.clone());
      }
    }

    // finally, check top words
    let mut dt = None;
    let mut beta = beta;
    for w in self.top_words(sd) {
      if beta <= 2 * alen as u32 {
        break;
      }
      let dt2 = self.solve_given(w, sd, beta);
      if let Some(dt2) = dt2 {
        if dt2.get_tot() < beta {
          beta = dt2.get_tot();
          dt = Some(dt2);
        }
      }
    }

    // add cache
    if !self.hard {
      if let Some(ref dt) = dt {
        sd.cache.add(self.clone(), dt.clone());
      }
    }

    dt
  }
}

impl<'a> Hash for State {
  fn hash<H: Hasher>(&self, h: &mut H) {
    self.n.hash(h);
    self.aws.hash(h);
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn check_news() {
    let (gwb, awb) = WBank::from2("/usr/share/hustle/bank1.csv", 5).unwrap();

    let state1 = State::new(gwb.data.clone(), awb.data.clone(), 5, false);
    let state2 = State::new2(gwb.data.clone(), awb.data.clone(), 5, 6, false);
    let state3 = State::new3();
    assert_eq!(state1, state2);
    assert_eq!(state2, state3);
  }

  // takes a while
  // #[test]
  fn simple_solve() {
    let mut sd = SData::new2(15);
    let state1 = State::new3();
    let mut state2 = State::new3();
    state2.hard = true;

    assert!(state1.solve(&mut sd, u32::MAX).is_some());
    assert!(state2.solve(&mut sd, u32::MAX).is_some());
  }

  #[test]
  fn impossible_solve() {
    let mut sd = SData::new2(2);
    let mut state = State::new3();
    state.n = 2;

    // cannot solve in 2 guesses
    assert!(state.solve(&mut sd, u32::MAX).is_none());
  }
}
