use std::iter::zip;
use std::hash::{Hash, Hasher};
use std::collections::{HashMap, HashSet};
use std::cmp;
use std::sync::Mutex;

use rand::prelude::*;
use rayon::prelude::*;

use super::analysis::HData;
use super::cache::Cache;
use crate::ds::*;

// TODO: also hash gws?
// could also iteratively hash when forming the state
type MFbMap<T> = HashMap<Vec<Feedback>, T>;

/// solve data
#[derive(Debug, Clone)]
pub struct MData {
  /// heuristic data
  pub hd: HData,
  /// cache
  pub cache: Cache,
  /// number of top guesses to try
  pub nguesses: u32,
  /// number of top answers to try
  pub nanswers: u32,
  /// number of remaining words makes it "endgame"
  pub endgcutoff: u32,
}

impl MData {
  pub fn new(hd: HData, cache: Cache, nguesses: u32,
             nanswers: u32, endgcutoff: u32) -> Self {
    Self {
      hd,
      cache,
      nguesses,
      nanswers,
      endgcutoff,
    }
  }

  pub fn new2(nguesses: u32, nanswers: u32) -> Self {
    let hd = HData::load(DEFHDP).unwrap();
    let cache = Cache::new(64, 8);
    Self::new(hd, cache, nguesses, nanswers, 15)
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MState {
  pub gws: Vec<Word>,
  pub awss: Vec<Vec<Word>>,
  pub wlen: u32,
  pub nwords: u32,
  pub turns: u32,
  pub finished: Vec<bool>,
  pub hard: bool,
}

pub fn fb_filter(gw: Word, fb: &Feedback, aws: &Vec<Word>) -> Vec<Word> {
  aws.iter()
    .cloned()
    .filter(|aw| *fb == Feedback::from(gw, *aw).unwrap())
    .collect()
}

pub fn fb_filter_all(gw: Word, fbs: &Vec<Feedback>, awss: &Vec<Vec<Word>>) -> Vec<Vec<Word>> {
  zip(fbs, awss)
    .map(|(fb, aws)| fb_filter(gw, fb, aws))
    .collect()
}

impl MState {
  pub fn new(gws: Vec<Word>, awss: Vec<Vec<Word>>,
             wlen: u32, nwords: u32, hard: bool) -> Self {
    MState {
      gws,
      awss,
      wlen,
      nwords,
      finished: vec![false; nwords as usize],
      turns: nwords + NEXTRA as u32,
      hard,
    }
  }

  pub fn new2(gws: Vec<Word>, awss: Vec<Vec<Word>>, wlen: u32,
              nwords: u32, finished: Vec<bool>, turns: u32, hard: bool) -> Self {
    MState {
      gws,
      awss,
      wlen,
      nwords,
      finished,
      turns,
      hard,
    }
  }

  pub fn new3() -> Self {
    let (gwb, awb) = WBank::from2("/usr/share/hustle/bank1.csv", NLETS as u8).unwrap();
    let gws = gwb.data;
    let awss = vec![awb.data];
    MState::new(gws, awss, 1, NLETS as u32, false)
  }

  pub fn size(&self) -> usize {
    self.awss.iter()
      .map(|aws| aws.len())
      .fold(1, |a, b| a*b)
  }

//  pub fn fb_filter_guesses(&self, gw: &Word, fbs: Vec<Feedback>) -> Vec<Word> {
//    if self.hard {
//      fbs.iter()
//        .map(|fb| HashSet::from_iter(fb_filter(*gw, *fb, &self.gws)))
//        .fold(HashSet::<Word>::new(), |a, b| (a | b))
//        .iter()
//        .cloned()
//        .collect()
//    } else {
//      self.gwss.clone()
//    }
//  }

  // get child state from guess and feedbacks
  pub fn fb_follow(&self, gw: Word, fbs: Vec<Feedback>) -> Self {
    let gws = self.gws.clone(); // for now
    let awss = fb_filter_all(gw, &fbs, &self.awss);
    let mut finished = zip(self.finished.clone(), fbs)
      .map(|(fin, fb)| fin || fb.is_correct())
      .collect();
    MState::new2(gws, awss, self.wlen, self.nwords, finished, self.turns - 1, self.hard)
  }

  pub fn sample_answers(&self, rng: &mut ThreadRng, md: &MData) -> Vec<Vec<Word>> {
    (0..md.nanswers as usize)
      .map(|_| {
        self.awss.iter()
          .map(|aws| aws.choose(rng).unwrap())
          .cloned()
          .collect()
      })
      .collect()
  }

  pub fn fb_partition(&self, gw: &Word, awss: Vec<Vec<Word>>) -> MFbMap<MState> {
    // for now just randomly access and make feedback as you go
    // TODO: use top-k NRA, LARA, etc?
    let mut fbp = Mutex::new(MFbMap::new());

    // iterate over sample answer lists
    awss.par_iter().for_each(|aws| {
      let fbs: Vec<Feedback> = aws.iter().map(|aw| Feedback::from(*gw, *aw).unwrap()).collect();
      if !fbp.lock().unwrap().contains_key(&fbs) {
        let gws2 = self.gws.clone(); // for now
        let awss2 = fb_filter_all(*gw, &fbs, &self.awss);
        let finished2 = zip(self.finished.clone(), fbs.clone()).map(|(fin, fb)| fin || fb.is_correct()).collect();
        let state = MState::new2(gws2, awss2, self.wlen, self.nwords, finished2, self.turns - 1, self.hard);
        
        let mut fbp = fbp.lock().unwrap();
        fbp.insert(fbs.clone(), state);
      }
    });

    fbp.into_inner().unwrap()
  }

  // get feedback counts for each column
  pub fn fb_counts(&self, gw: &Word) -> Vec<HashMap<Feedback, u32>> {
    self.awss.iter().map(|aws| {
      let mut map = HashMap::new();
      for aw in aws {
        let fb = Feedback::from(*gw, *aw).unwrap();
        *map.entry(fb).or_insert(0) += 1;
      }
      map
    }).collect()
  }
  
  // approximately quantify how good each guess is
  // TODO bad for low numbers, add bonus for potentially correct guess
  pub fn heuristic(&self, gw: &Word, md: &MData) -> f64 {
    (self.fb_counts(gw), &self.finished)
      .into_par_iter()
      .map(|(fbc, &fin)| {
        if fin { return 0. }

        let mut tot = 0f64;
        let mut sz = 0;
        for (fb, n) in fbc {
          if !fb.is_correct() {
            tot += md.hd.get_approx(n as usize);
          }
          sz += n;
        }
        tot / sz as f64
      })
      .sum()
  }

  pub fn top_words(&self, md: &MData) -> Vec<Word> {
    let mut tups: Vec<(Word, f64)> = self
      .gws.clone()
      .into_par_iter()
      .map(|gw| (gw, self.heuristic(&gw, md)))
      .collect();
    tups.sort_by(|(_, f1), (_, f2)| f1.partial_cmp(f2).unwrap());
    tups
      .iter()
      .map(|(gw, _)| *gw)
      .take(md.nguesses as usize)
      .collect()
  }

  pub fn solve_given(&self, gw: Word, md: &mut MData) -> Option<f64> {
    let awss_sample = self.sample_answers(&mut rand::thread_rng(), md);
    let fbps = self.fb_partition(&gw, awss_sample);

    let mut tot = 0.;
    let mut sz = 0;
    for (fb, state) in fbps.iter() {
      let sz2 = state.size();
      tot += sz2 as f64 * state.solve(md)?;
      sz += sz2;
    }

    Some(1. + tot / sz as f64)
  }

  pub fn solve(&self, md: &mut MData) -> Option<f64> {
    if self.finished.iter().all(|&fin| fin) {return Some(0.)}
    if self.turns == 0 {return None}

    let n_finished: usize = self.finished.iter().map(|&fin| fin as usize).sum();
    let n_unfinished: usize = self.nwords as usize - n_finished;


    // one answer -> guess it
    for (aws, fin) in zip(&self.awss, &self.finished) {
      if aws.len() == 1 && !fin {
        return self.solve_given(*aws.get(0).unwrap(), md);
      }
    }

    // check if a potential answer fixes rest
    if self.awss.iter().all(|aws| aws.len() < 15) {
      let mut smallest_fix = usize::MAX;
      for (aws, fin) in zip(&self.awss, &self.finished) {
        if aws.len() >= smallest_fix || *fin {continue}
        for aw in aws {
          if self.fb_counts(&aw).iter().all(|fbc| fbc.iter().all(|(fb, ct)| *ct == 1)) {
            smallest_fix = aws.len();
          }
        }
      }

      if smallest_fix != usize::MAX {
        return Some(n_unfinished as f64 - 1f64 / smallest_fix as f64);
      }
    }

    // check cache
//    if !self.hard {
//      if let Some(dt) = md.cache.read(self) {
//        return Some(dt.clone());
//      }
//    }

    // find best top word
    let mut tot = f64::INFINITY;
    let tops = self.top_words(md);
    for w in tops {
      let tot2 = self.solve_given(w, md);
      if let Some(tot2) = self.solve_given(w, md) {
        if tot2 < tot {tot = tot2}
        // return if best case
        if tot2 == n_unfinished as f64 {return Some(tot2)}
      }
    }

    // add cache
//    if !self.hard {
//      if let Some(ref dt) = dt {
//        md.cache.add(self.clone(), dt.clone());
//      }
//    }

    if tot == f64::INFINITY { None } else { Some(tot) }
  }
}

impl<'a> Hash for MState {
  fn hash<H: Hasher>(&self, h: &mut H) {
    self.gws.hash(h);
    self.awss.hash(h);
    self.finished.hash(h);
    self.wlen.hash(h);
    self.nwords.hash(h);
    self.turns.hash(h);
    self.hard.hash(h);
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn solve_blah() {
    let (gwb, awb) = WBank::from2("/usr/share/hustle/bank1.csv", 5).unwrap();
    let mut state = MState::new(gwb.data, vec![awb.data; 6], 5, 6, false);
    let mut md = MData::new2(7, 5);

    state = state.fb_follow(Word::from_str("salet").unwrap(), vec![
      Feedback::from_str("bgbgb").unwrap(),
      Feedback::from_str("byybb").unwrap(),
      Feedback::from_str("bbybb").unwrap(),
      Feedback::from_str("bgbgb").unwrap(),
      Feedback::from_str("byyyb").unwrap(),
      Feedback::from_str("bbbyb").unwrap(),
    ]);
//    state = state.fb_follow(Word::from_str("brick").unwrap(), vec![
//      Feedback::from_str("gbbyb").unwrap(),
//      Feedback::from_str("byybb").unwrap(),
//    ]);
//    state = state.fb_follow(Word::from_str("podgy").unwrap(), vec![
//      Feedback::from_str("bybbb").unwrap(),
//      Feedback::from_str("bbbbb").unwrap(),
//    ]);

    let tot = state.solve(&mut md);
    println!("tot: {:?}", tot);
    assert!(tot.is_some());
    assert!(false);
  }

  #[test]
  fn solve_endgame() {
    let (gwb, awb) = WBank::from2("/usr/share/hustle/bank1.csv", 5).unwrap();
    let mut state = MState::new(gwb.data, vec![awb.data; 2], 5, 2, false);
    let mut md = MData::new2(0, 0);

    // FLICK, ICILY
    // ENSUE, GUESS, GUISE, ISSUE
    // guessing ENSUE/ISSUE gives 1.75
    // which should be found in endgame check
    state = state.fb_follow(Word::from_str("salet").unwrap(), vec![
      Feedback::from_str("bbybb").unwrap(),
      Feedback::from_str("ybbyb").unwrap(),
    ]);
    state = state.fb_follow(Word::from_str("courd").unwrap(), vec![
      Feedback::from_str("ybbbb").unwrap(),
      Feedback::from_str("bbybb").unwrap(),
    ]);
    assert!(state.solve(&mut md) == Some(1.75));
  }

//  #[test]
//  fn check_news() {
//    let (gwb, awb) = WBank::from2("/usr/share/hustle/bank1.csv", 5).unwrap();
//
//    let state1 = MState::new(gwb.data.clone(), awb.data.clone(), 5, false);
//    let state2 = MState::new2(gwb.data.clone(), awb.data.clone(), 5, 6, false);
//    let state3 = MState::new3();
//    assert_eq!(state1, state2);
//    assert_eq!(state2, state3);
//  }

  // takes a while
  // #[test]
//  fn simple_solve() {
//    let mut md = MData::new2(15);
//    let state1 = MState::new3();
//    let mut state2 = MState::new3();
//    state2.hard = true;
//
//    assert!(state1.solve(&mut md, u32::MAX).is_some());
//    assert!(state2.solve(&mut md, u32::MAX).is_some());
//  }

//  #[test]
//  fn impossible_solve() {
//    let mut md = MData::new2(2);
//    let mut state = MState::new3();
//    state.n = 2;
//
//    // cannot solve in 2 guesses
//    assert!(state.solve(&mut md, u32::MAX).is_none());
//  }
}
