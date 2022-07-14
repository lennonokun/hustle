use termion::color;

use super::gameio::GameIO;
use super::play::PlayScreen;
use super::config::Config;
use crate::ds::*;

#[derive(Debug)]
pub struct FeedbackCol {
  pub aw: Word,
  pub feedbacks: Vec<Feedback>,
  pub guesses: Vec<Word>,
  pub wlen: u8,
  pub done: bool,
}

fn fb_color_index(fb: &Feedback, i: usize) -> usize {
  if fb.get_g(i as u8) {
    2
  } else if fb.get_y(i as u8) {
    1
  } else {
    0
  }
}

impl FeedbackCol {
  /// create new feedback col with answer aw
  pub fn new(aw: Word) -> Self {
    Self {
      aw,
      feedbacks: Vec::<Feedback>::new(),
      guesses: Vec::<Word>::new(),
      wlen: aw.wlen,
      done: false,
    }
  }

  /// guess and return if newly done
  pub fn guess(&mut self, gw: Word) -> bool {
    if self.done || gw.wlen != self.aw.wlen {
      return false;
    }
    let fb = Feedback::from(gw, self.aw).unwrap();
    self.guesses.push(gw);
    self.feedbacks.push(fb);
    self.done = fb.is_correct();
    self.done
  }

  /// get display string for row i or None
  pub fn get(&self, i: usize, cfg: &Config) -> Option<String> {
    let gw = self.guesses.get(i)?;
    let fb = self.feedbacks.get(i)?;
    let mut out = cfg.fb_fg.fg_string();
    for j in 0..self.wlen as usize {
      let idx = fb_color_index(fb, j);
      out.push_str(cfg.fb_bgs[idx].bg_string().as_str());
      out.push(gw.get(j)?);
    }
    out.push_str(color::Reset.fg_str());
    out.push_str(color::Reset.bg_str());
    Some(out)
  }
}
