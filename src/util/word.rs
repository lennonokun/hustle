use std::fmt;

use super::misc::MAXWLEN;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Word {
  pub data: [u8; MAXWLEN],
  pub wlen: u8,
}

impl Word {
  pub fn from(s: String) -> Option<Self> {
    let wlen = s.len() as u8;
    let mut data = [0u8; MAXWLEN];
    if s.len() > MAXWLEN {
      return None;
    }
    for (i, c) in s.to_ascii_uppercase().chars().enumerate() {
      data[i] = c as u8 - b'A';
    }
    Some(Word { data, wlen })
  }

  pub fn from_str(s: &str) -> Option<Self> {
    let wlen = s.len() as u8;
    let mut data = [0; MAXWLEN];
    if s.len() > MAXWLEN {
      return None;
    }
    for (i, c) in s.to_ascii_uppercase().chars().enumerate() {
      data[i] = c as u8 - b'A';
    }
    Some(Word { data, wlen })
  }

  pub fn get(&self, i: usize) -> Option<char> {
    if i >= self.wlen.into() {return None}
    Some((self.data[i] + b'A') as char)
  }

  pub fn to_string(&self) -> String {
    self.data[0..self.wlen as usize]
      .iter()
      .cloned()
      .map(|x| (x + b'A') as char)
      .collect()
  }
}

impl fmt::Display for Word {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

impl fmt::Debug for Word {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  const WORD_STRINGS: [&'static str; 5] = [
    "LODGE", "humongous", "DAnG", "enormOUS", "TOXICITY"
  ];

  #[test]
  fn to_from() {
    for s in WORD_STRINGS {
      let w = Word::from_str(s).unwrap();
      let s2 = s.to_ascii_uppercase();
      assert_eq!(s2, w.to_string());
      assert_eq!(s2.len(), w.wlen as usize);
    }
  }

  #[test]
  fn format() {
    for s in WORD_STRINGS {
      let w = Word::from_str(s).unwrap();
      let s2 = s.to_ascii_uppercase();
      assert_eq!(s2, format!("{}", w));
      assert_eq!(s2, format!("{:?}", w));
    }
  }
  
  #[test]
  fn get() {
    for s in WORD_STRINGS {
      let w = Word::from_str(s).unwrap();
      let s2 = s.to_ascii_uppercase();
      for (i, c) in s2.chars().enumerate() {
        assert_eq!(Some(c), w.get(i));
      }
      assert_eq!(None, w.get(s.len()));
    }
  }
}
