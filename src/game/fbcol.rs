use termion::color;

use super::config::Config;
use crate::ds::*;

#[derive(Debug)]
pub struct FeedbackCol {
  pub ans: Word,
  pub rows: Vec<String>,
  pub wlen: u8,
  pub done: bool,
}

impl FeedbackCol {
  pub fn new(ans: Word) -> Self {
    Self {
      ans,
      rows: Vec::<String>::new(),
      wlen: ans.wlen,
      done: false,
    }
  }

  // returns if newly finished
  pub fn guess(&mut self, gw: Word, cfg: &Config) -> bool {
    if self.done || self.wlen != gw.wlen {
      return false;
    }
    let fb = Feedback::from(gw, self.ans).unwrap();
    let mut s = String::new();
    for i in 0..self.wlen {
      let fg = cfg.fb_fg;
      let bg = cfg.fb_bgs[if fb.get_g(i) {2} else if fb.get_y(i) {1} else {0}];
      s += &format!("{}{}", fg.fg_string(), bg.bg_string());
      s.push((gw.data[i as usize] + b'A') as char);
    }
    s += &format!("{}{}", color::Reset.fg_str(), color::Reset.bg_str());
    self.rows.push(s);
    self.done = gw == self.ans;
    self.done
  }
}
