use termion::color;

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
  pub fn guess(&mut self, gw: Word) -> bool {
    if self.done || self.wlen != gw.wlen {
      return false;
    }
    let fb = Feedback::from(gw, self.ans).unwrap();
    let mut s = String::new();
    let fg_color = color::White.fg_str();
    for i in 0..self.wlen {
      let bg_color = if fb.get_g(i) {
        color::Green.bg_str()
      } else if fb.get_y(i) {
        color::Yellow.bg_str()
      } else {
        color::Blue.bg_str()
      };

      s += &format!("{}{}", fg_color, bg_color);
      s.push((gw.data[i as usize] + b'A') as char);
    }
    s += &format!("{}{}", color::Reset.fg_str(), color::Reset.bg_str());
    self.rows.push(s);
    self.done = gw == self.ans;
    self.done
  }
}
