use std::iter::zip;
use std::hash::{Hash, Hasher};
use std::collections::{HashMap};
use std::sync::{Arc, Mutex};
use std::cmp::{min, Ordering};

use rand::prelude::*;
use rayon::prelude::*;
use pdqselect::select_by;

use super::cache::Cache;
use crate::util::*;

// TODO: also hash gws?
// could also iteratively hash when forming the state
type MFbMap<T> = HashMap<Vec<Feedback>, T>;

/// solve data
#[derive(Debug, Clone)]
pub struct MData {
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
  pub fn new(cache: Cache, nguesses: u32,
             nanswers: u32, endgcutoff: u32) -> Self {
    Self {
      cache,
      nguesses,
      nanswers,
      endgcutoff,
    }
  }

  pub fn new2(nguesses: u32, nanswers: u32) -> Self {
    let cache = Cache::new(64, 8);
    Self::new(cache, nguesses, nanswers, 15)
  }
//
//  pub fn deep_clone(&self) -> Self {
//    let cache2 = (*self.cache.lock().unwrap()).clone();
//    Self::new(cache2, self.nguesses, self.nanswers, self.ecut)
//  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MState {
  pub gws: Arc<Vec<Word>>,
  pub awss: Vec<Vec<Word>>,
  pub finished: Vec<bool>,
  pub turns: u32,
  pub hard: bool,
  pub wlen: u8,
  pub nwords: u32,
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
  pub fn new(wbank: &WBank, nwords: u32, turns: Option<u32>, hard: bool) -> Self {
    Self {
      gws: Arc::new(wbank.gws.clone()),
      awss: vec![wbank.aws.clone(); nwords as usize],
      finished: vec![false; nwords as usize],
      turns: turns.unwrap_or(nwords + NEXTRA as u32),
      hard,
      wlen: wbank.wlen,
      nwords,
    }
  }

  pub fn child(&self, gws: Arc<Vec<Word>>, awss: Vec<Vec<Word>>, finished: Vec<bool>) -> Self {
    Self {
      gws,
      awss,
      finished,
      turns: self.turns - 1,
      hard: self.hard,
      wlen: self.wlen,
      nwords: self.nwords,
    }
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
    let finished = zip(self.finished.clone(), fbs)
      .map(|(fin, fb)| fin || fb.is_correct())
      .collect();
    self.child(gws, awss, finished)
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
    let fbp = Mutex::new(MFbMap::new());

    // iterate over sample answer lists
    awss.par_iter().for_each(|aws| {
      let fbs: Vec<Feedback> = aws.iter().map(|aw| Feedback::from(*gw, *aw).unwrap()).collect();
      if !fbp.lock().unwrap().contains_key(&fbs) {
        let gws = self.gws.clone();
        let awss = fb_filter_all(*gw, &fbs, &self.awss);
        let finished = zip(self.finished.clone(), fbs.clone())
          .map(|(fin, fb)| fin || fb.is_correct())
          .collect();
        let state = self.child(gws, awss, finished);
        
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
  pub fn heuristic(&self, gw: &Word) -> f32 {
    (&self.awss, &self.finished)
      .into_par_iter()
      .map(|(aws, &fin)| {
        if fin { return 0. }

        let mut parts = vec![false; 3usize.pow(self.wlen as u32)];
        let mut sum = 0;

        for aw in aws.iter().cloned() {
          let i = fb_id(*gw, aw) as usize;
          if !parts[i] {
            sum += 1;
            parts[i] = true;
          }
        }

        // slow-ish
        if aws.contains(gw) {
          (16*sum + 1) as f32 / aws.len() as f32
        } else {
          (16*sum) as f32 / aws.len() as f32
        }
      })
      .sum()
  }

  pub fn top_words(&self, md: &MData) -> Vec<Word> {
    #[derive(Debug, Clone, Copy)]
    struct ScoredWord {
      pub w: Word,
      pub h: f32,
    }

    // reversed, we want to select the highest h, not lowest
    fn cmp_scored(sw1: &ScoredWord, sw2: &ScoredWord) -> Ordering {
      (&sw2.h).partial_cmp(&sw1.h).unwrap()
    }

    let glen = self.gws.len();
    let ntops = min(md.nguesses as usize, glen);

    // select ntops with slow heuristic
    let mut tops: Vec<ScoredWord> = (&self.gws)
      .par_iter()
      .map(|gw| ScoredWord {w: *gw, h: self.heuristic(&gw)})
      .collect();
    select_by(&mut tops, ntops-1, &mut cmp_scored);

    tops.iter().take(ntops).map(|tw| tw.w).collect()
  }

  pub fn solve_given(&self, gw: Word, md: &mut MData) -> Option<f32> {
    let awss_sample = self.sample_answers(&mut rand::thread_rng(), md);
    let fbps = self.fb_partition(&gw, awss_sample);

    let mut tot = 0.;
    let mut sz = 0;
    for (_fb, state) in fbps.iter() {
      let sz2 = state.size();
      tot += sz2 as f32 * state.solve(md)?;
      sz += sz2;
    }

    Some(1. + tot / sz as f32)
  }

  pub fn solve(&self, md: &mut MData) -> Option<f32> {
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
          if self.fb_counts(&aw).iter().all(|fbc| fbc.iter().all(|(_fb, ct)| *ct == 1)) {
            smallest_fix = aws.len();
          }
        }
      }

      if smallest_fix != usize::MAX {
        return Some(n_unfinished as f32 - 1f32 / smallest_fix as f32);
      }
    }

    // check cache
//    if !self.hard {
//      if let Some(dt) = md.cache.read(self) {
//        return Some(dt.clone());
//      }
//    }

    // find best top word
    let mut tot = f32::INFINITY;
    let tops = self.top_words(md);
    for w in tops {
      if let Some(tot2) = self.solve_given(w, md) {
        if tot2 < tot {tot = tot2}
        // return if best case
        if tot2 == n_unfinished as f32 {return Some(tot2)}
      }
    }

    // add cache
//    if !self.hard {
//      if let Some(ref dt) = dt {
//        md.cache.add(self.clone(), dt.clone());
//      }
//    }

    if tot == f32::INFINITY { None } else { Some(tot) }
  }
}

impl Hash for MState {
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
    let wbank = WBank::load1().unwrap();
    let mut state = MState::new(&wbank, 6, None, false);
    let mut md = MData::new2(5, 5);

    state = state.fb_follow(Word::from_str("salet").unwrap(), vec![
      Feedback::from_str("bgbgb").unwrap(),
      Feedback::from_str("byybb").unwrap(),
      Feedback::from_str("bbybb").unwrap(),
      Feedback::from_str("bgbgb").unwrap(),
      Feedback::from_str("byyyb").unwrap(),
      Feedback::from_str("bbbyb").unwrap(),
    ]);
    state = state.fb_follow(Word::from_str("courd").unwrap(), vec![
      Feedback::from_str("bbbbb").unwrap(),
      Feedback::from_str("bbbbb").unwrap(),
      Feedback::from_str("bbbbb").unwrap(),
      Feedback::from_str("bbbbb").unwrap(),
      Feedback::from_str("bbbbb").unwrap(),
      Feedback::from_str("bbbbb").unwrap(),
    ]);

    let tot = state.solve(&mut md);
    assert!(tot.is_some());
  }

  #[test]
  fn solve_endgame() {
    let wbank = WBank::load1().unwrap();
    let mut state = MState::new(&wbank, 2, None, false);
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
}

