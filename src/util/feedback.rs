use std::fmt;
use std::collections::HashMap;

use super::word::Word;
use super::misc::MAXWLEN;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Feedback {
  // green + yellow bitsets
  g_bs: u16,
  y_bs: u16,
  wlen: u8,
}

pub type FbMap<T> = HashMap<Feedback, T>;

pub fn fb_id(mut gw: Word, mut aw: Word) -> u32 {
  let wlen = gw.wlen as usize;
  let mut id = 0;

  let mut w = 1;
  for i in 0..wlen {
    if gw.data[i] == aw.data[i] {
      id += 2*w;
      gw.data[i] = 254;
      aw.data[i] = 255;
    }
    w *= 3;
  }

  let mut w = 1;
  for i in 0..wlen {
    for j in 0..wlen {
      if gw.data[i] == aw.data[j] {
        id += w;
        aw.data[j] = 255;
        break;
      }
    }
    w *= 3;
  }

  id
}

impl Feedback {
  pub fn from(mut gw: Word, mut aw: Word) -> Option<Self> {
    if gw.wlen != aw.wlen {
      return None;
    }
    let wlen = gw.wlen;
    let mut g_bs = 0u16;
    let mut y_bs = 0u16;
    // first find greens
    for i in 0..wlen as usize {
      if gw.data[i] == aw.data[i] {
        g_bs |= 1 << i;
        // remove
        gw.data[i] = 255;
        aw.data[i] = 255;
      }
    }
    // then find yellows
    for i in 0..wlen as usize {
      if gw.data[i] < 255 {
        for j in 0..wlen as usize {
          if gw.data[i] == aw.data[j] {
            y_bs |= 1 << i;
            gw.data[i] = 255;
            aw.data[j] = 255;
            break;
          }
        }
      }
    }
    Some(Feedback { g_bs, y_bs, wlen })
  }

  pub fn from_id(mut id: u32, wlen: u8) -> Self {
    let mut g_bs = 0u16;
    let mut y_bs = 0u16;

    for i in 0..wlen {
      let r = id % 3;
      id = id / 3;

      if r == 2 {
        g_bs |= 1 << i;
      } else if r == 1{
        y_bs |= 1 << i;
      }
    }

    Feedback { g_bs, y_bs, wlen }
  }

  pub fn to_id(&self) -> u32 {
    let mut id = 0;
    let mut w = 1;
    for i in 0..self.wlen {
      if self.get_g(i) {
        id += 2*w;
      } else if self.get_y(i) {
        id += w;
      }
      w *= 3;
    }
    id
  }

  pub fn to_string(&self) -> String {
    let mut out = String::new();
    for i in 0..self.wlen {
      if self.g_bs & 1 << i != 0 {
        out.push('G');
      } else if self.y_bs & 1 << i != 0 {
        out.push('Y');
      } else {
        out.push('B');
      }
    }
    out
  }

  pub fn from_str(s: &str) -> Option<Self> {
    let wlen = s.len() as u8;
    if wlen > MAXWLEN as u8 {
      return None;
    }
    let mut fb = Feedback {
      g_bs: 0,
      y_bs: 0,
      wlen,
    };
    for (i, c) in s
      .to_ascii_uppercase()
      .chars()
      .take(wlen as usize)
      .enumerate()
    {
      if c == 'G' {
        fb.g_bs |= 1 << i;
      } else if c == 'Y' {
        fb.y_bs |= 1 << i;
      }
    }
    Some(fb)
  }

  pub fn get_g(&self, i: u8) -> bool {
    self.g_bs & 1 << i != 0
  }

  pub fn get_y(&self, i: u8) -> bool {
    self.y_bs & 1 << i != 0
  }

  pub fn is_correct(&self) -> bool {
    self.g_bs == ((1 << self.wlen) - 1)
  }
}

impl fmt::Display for Feedback {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

impl fmt::Debug for Feedback {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  const FB_STRINGS: [&'static str; 5] = [
    "bbYBG", "GyBgBBby", "YYGY", "BBYGYB", "yyGYBby"
  ];

  #[test]
  fn to_from() {
    let w1 = Word::from_str("salve").unwrap();
    let w2 = Word::from_str("raise").unwrap();
    let w3 = Word::from_str("cabal").unwrap();
    let w4 = Word::from_str("antic").unwrap();

    let fb1 = Feedback::from(w1, w2).unwrap();
    let fb2 = Feedback::from(w3, w4).unwrap();
    let id1 = fb_id(w1, w2);
    let id2 = fb_id(w3, w4);

    assert_eq!(fb1, Feedback::from_str("ygbbg").unwrap());
    assert_eq!(fb2, Feedback::from_str("yybbb").unwrap());
    assert_eq!(id1, 1*1 + 2*3 + 2*81);
    assert_eq!(id2, 1*1 + 1*3);   

    assert_eq!(fb1, Feedback::from_id(id1, 5));
    assert_eq!(fb2, Feedback::from_id(id2, 5));
    assert_eq!(id1, fb1.to_id());
    assert_eq!(id2, fb2.to_id());
  }

  #[test]
  fn format() {
    for s in FB_STRINGS {
      let fb = Feedback::from_str(s).unwrap();
      let s2 = s.to_ascii_uppercase();
      assert_eq!(s2, format!("{}", fb));
      assert_eq!(s2, format!("{:?}", fb));
    }
  }
}
