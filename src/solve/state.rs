use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use rand::Rng;
use rayon::prelude::*;

use super::cache::Cache;
use super::adata::AData;
use crate::util::*;

// TODO: also hash gws?
// could also iteratively hash when forming the state

// maximum number of words solveable in two guesses
const MAX_TWOSOLVE: u32 = 20;

/// solve data
#[derive(Debug, Clone)]
pub struct SData {
  /// analysis data
  pub adata: AData,
  /// cache
  pub cache: Arc<Mutex<Cache>>,
  /// number of top words to try using soft heuristic
  pub ntops1: u32,
  /// number of top words to try using hard heuristic
  pub ntops2: u32,
  /// number of remaining words makes it "endgame"
  pub ecut: u32,
}

impl SData {
  pub fn new(adata: AData, cache: Cache, ntops1: u32,
             ntops2: u32, ecut: u32) -> Self {
    let cache = Arc::new(Mutex::new(cache));
    Self {
      adata,
      cache,
      ntops1,
      ntops2,
      ecut,
    }
  }

  pub fn new2(ntops1: u32, ntops2: u32) -> Self {
    let adata = AData::load(DEFHDP, DEFLDP).unwrap();
    let cache = Cache::new(64, 8);
    Self::new(adata, cache, ntops1, ntops2, 15)
  }
}

#[derive(Clone)]
struct GivenData {
  pub dt: Option<DTree>,
  pub beta: u32,
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
    let mut cts = vec![0; 3usize.pow(self.wlen)];

    for aw in &self.aws {
      cts[fb_id(*gw, *aw) as usize] += 1;
    }

    cts
  }

  pub fn letter_evals(&self) -> (Vec<Vec<f64>>, Vec<f64>) {
    // get letter counts
    let mut gss = vec![vec![0usize; self.wlen as usize]; 26];
    let mut ys = vec![0usize; 26];
    for aw in &self.aws {
      for i in 0..(self.wlen as usize) {
        gss[aw.data[i] as usize][i] += 1;
        if !aw.data[0..i].contains(&aw.data[i]) {
          ys[aw.data[i] as usize] += 1;
        }
      }
    }

    // maximize entropy (very fuzzy) 
    let n = self.aws.len() as f64;
    let gss = gss.iter()
      .map(|gs| gs.iter()
           .map(|&g| {let p = (g as f64) / n; p*(1.-p)})
           .collect())
      .collect();
    let ys = ys.iter()
      .map(|&y| {let p = (y as f64) / n; p*(1.-p)})
      .collect();
    (gss, ys)
  }

  pub fn letter_heuristic(&self, gw: &Word, gss: &Vec<Vec<f64>>, ys: &Vec<f64>) -> f64 {
    let mut h = 0f64;
    for i in 0..(self.wlen as usize) {
      h += gss[gw.data[i] as usize][i];
      if !gw.data[0..i].contains(&gw.data[i]) {
        h += ys[gw.data[i] as usize];
      }
    }

    if self.aws.contains(&gw) {
      h * 1.05
    } else {
      h
    }
  }

  pub fn heuristic(&self, gw: &Word, sd: &SData) -> f64 {
    let h = if self.wlen <= 5 {
      self.fb_counts_vec(gw)
        .iter()
        .filter(|x| **x > 0)
        .map(|&x| sd.adata.get_approx(x as usize).unwrap())
        .sum()
    } else {
      self.fb_counts(gw)
        .iter()
        .map(|(_, n)| sd.adata.get_approx(*n as usize).unwrap())
        .sum()
    };

    if self.aws.contains(gw) {
      h - 1.
    } else {
      h
    }
  }

  pub fn top_words(&self, sd: &SData) -> Vec<Word> {
    // fast heuristic
    let (gss, ys) = self.letter_evals();
    let mut tups: Vec<(Word, f64)> = self
      .gws.clone()
      .into_par_iter()
      .map(|gw| (gw, self.letter_heuristic(&gw, &gss, &ys)))
      .collect();
    tups.sort_by(|(_, f1), (_, f2)| f2.partial_cmp(f1).unwrap());
    let gws2 = tups.iter()
      .take(sd.ntops1 as usize)
      .map(|(gw, _)| *gw)
      .collect::<Vec<Word>>();
    
    // slow heuristic
    let mut tups: Vec<(Word, f64)> = gws2
      .iter()
      .map(|gw| (*gw, self.heuristic(&gw, sd)))
      .collect();
    tups.sort_by(|(_, f1), (_, f2)| f1.partial_cmp(f2).unwrap());
    tups
      .iter()
      .map(|(gw, _)| *gw)
      .take(if self.hard {2 * sd.ntops2 as usize} else {sd.ntops2 as usize})
      .collect()
  }

  pub fn solve_given(&self, gw: Word, sd: &SData, beta: u32) -> Option<DTree> {
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

  pub fn solve(&self, sd: &SData, beta: u32) -> Option<DTree> {
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
    if alen <= sd.ecut as usize {
      for aw in self.aws.iter() {
        if self.fb_counts(aw).values().all(|c| *c == 1) {
          return self.solve_given(*aw, sd, beta);
        }
      }
    }
    // check cache
    if !self.hard {
      let mut cache = sd.cache.lock().unwrap();
      if let Some(dt) = cache.read(self) {
        return Some(dt.clone());
      }
    }

    // finally, check top words
    let tws = self.top_words(&sd);
    let gd = Mutex::new(GivenData{dt: None, beta});
    tws.into_par_iter().for_each(|w| {
      let gd2 = gd.lock().unwrap().clone();
      if gd2.beta <= 2 * alen as u32 {return}
      let dt2 = self.solve_given(w, sd, gd2.beta);
      if let Some(dt2) = dt2 {
        let mut gd = gd.lock().unwrap();
        if dt2.get_tot() < gd.beta {
          gd.beta = dt2.get_tot();
          gd.dt = Some(dt2);
        }
      }
    });

    let gd = gd.into_inner().unwrap();
    let dt = gd.dt;

    // add cache
    if !self.hard {
      if let Some(ref dt) = dt {
        let mut cache = sd.cache.lock().unwrap();
        cache.add(self.clone(), dt.clone());
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

  #[test]
  fn simple_solve() {
    let sd = SData::new2(1000, 10);
    let state1 = State::new3();
    let mut state2 = State::new3();
    state2.hard = true;

    assert!(state1.solve(&sd, u32::MAX).is_some());
    assert!(state2.solve(&sd, u32::MAX).is_some());
  }

  #[test]
  fn impossible_solve() {
    let mut sd = SData::new2(200, 2);
    let mut state = State::new3();
    state.n = 2;

    // cannot solve in 2 guesses
    assert!(state.solve(&sd, u32::MAX).is_none());
  }
}
