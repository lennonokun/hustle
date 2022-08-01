use core::str::FromStr;
use std::fmt;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write, Error, ErrorKind};
use std::path::Path;

use rand::prelude::*;
use rand::distributions::Distribution;
use rand::distributions::uniform::{Uniform, SampleUniform};
use rayon::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

pub const NLETS: usize = 5;
pub const NGUESSES: usize = 6;
pub const NEXTRA: usize = 5;
pub const NWORDS: usize = 2309;
pub const MINWLEN: usize = 4;
pub const MAXWLEN: usize = 11;

pub const DEFWBP: &'static str = "/usr/share/hustle/bank1.csv";
pub const DEFHDP: &'static str = "/usr/share/hustle/happrox.csv";

pub fn is_alpha(c: char) -> bool {
  ('a'..='z').contains(&c) || ('A'..='Z').contains(&c)
}

// assumes c is alpha 
pub fn upper(c: char) -> char {
  if ('a'..='z').contains(&c) {
    (c as u8 + b'A' - b'a') as char
  } else {
    c
  }
}

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
    if i > self.wlen.into() {return None}
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

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Feedback {
  // green + yellow bitsets
  g_bs: u16,
  y_bs: u16,
  wlen: u8,
}

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

#[derive(Debug, Clone)]
pub struct WBank {
  pub data: Vec<Word>,
  pub wlen: u8,
}

impl WBank {
  pub fn from2<P>(p: P, wlen: u8) -> io::Result<(Self, Self)>
  where
    P: AsRef<Path>, {
    let file = File::open(p)?;
    let reader = BufReader::new(file);
    let mut gdata = Vec::<Word>::new();
    let mut adata = Vec::<Word>::new();
    for line in reader.lines().skip(1).flatten() {
      // parse line
      let vec: Vec<&str> = line.split(',').collect();
      if vec[2].parse::<u8>().unwrap() != wlen {
        continue;
      }
      // push to both if answer word, but only guess if guess word
      let w = Word::from_str(vec[0]).unwrap();
      if vec[1] == "A" {
        adata.push(w)
      }
      gdata.push(w);
    }

    Ok((WBank { data: gdata, wlen }, WBank { data: adata, wlen }))
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn new() -> Self {
    WBank {
      data: Vec::new(),
      wlen: 0,
    }
  }

  pub fn new2(wlen: u8) -> Self {
    WBank {
      data: Vec::new(),
      wlen,
    }
  }

  pub fn contains(&self, w: Word) -> bool {
    self.data.contains(&w)
  }

  pub fn pick(&self, rng: &mut ThreadRng, n: usize) -> Vec<Word> {
    self.data.choose_multiple(rng, n).cloned().collect()
  }

  pub fn to_string(&self) -> String {
    let s = self
      .data
      .iter()
      .map(|w| w.to_string())
      .collect::<Vec<String>>()
      .join(" ");
    format!("[{s}]")
  }
}

pub type FbMap<T> = HashMap<Feedback, T>;

// decision tree
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DTree {
  Leaf,
  Node {
    // total leaf depth
    tot: u32,
    // word
    word: Word,
    // children per unique feedback
    fbmap: FbMap<DTree>,
  },
}

impl DTree {
  pub fn follow(&self, fb: Feedback) -> Option<&DTree> {
    match self {
      DTree::Leaf => None,
      DTree::Node {
        tot: _,
        word: _,
        fbmap,
      } => fbmap.get(&fb),
    }
  }

  pub fn get_tot(&self) -> u32 {
    match self {
      DTree::Leaf => 0,
      DTree::Node {
        tot,
        word: _,
        fbmap: _,
      } => *tot,
    }
  }

  pub fn get_fbmap(&self) -> Option<&FbMap<DTree>> {
    match self {
      DTree::Leaf => None,
      DTree::Node {
        tot: _,
        word: _,
        fbmap,
      } => Some(fbmap),
    }
  }

  pub fn pprint<W>(&self, out: &mut W, indent: &String, n: u32)
  where
    W: Write, {
    match self {
      DTree::Leaf => {}
      DTree::Node { tot, word, fbmap } => {
        writeln!(out, "{}{}, {}", indent, word.to_string(), tot);
        let mut indent2 = indent.clone();
        indent2.push(' ');
        let mut items: Vec<(&Feedback, &DTree)> = fbmap.iter().collect();
        items.sort_by_key(|(fb, dt)| fb.to_id());
        for (fb, dt) in items {
          writeln!(out, "{}{}{}", indent2, fb.to_string(), n);
          dt.pprint(out, &indent2, n + 1);
        }
      }
    }
  }
}

pub struct Range<X> where X: Copy + SampleUniform {
  /// lower bound
  pub a: X,
  /// upper bound
  pub b: X,
  /// inclusive flag
  pub inc: bool,
  /// distribution
  pub dist: Uniform<X>,
}

impl<X> Range<X> where X: Copy + SampleUniform {
  pub fn new(a: X, b: X, inc: bool) -> Self {
    let dist = if inc {
      Uniform::new_inclusive(a, b)
    } else {
      Uniform::new(a, b)
    };
    Self {a, b, inc, dist}
  }

  pub fn sample(&self, rng: &mut ThreadRng) -> X {
    self.dist.sample(rng)
  }
}

impl<X> FromStr for Range<X>
where X: Copy + FromStr + SampleUniform {
  type Err = Error;
  
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    lazy_static! {
      static ref RE_UNIF: Regex = Regex::new(r"^(\d+)..(=?)(\d+)$").unwrap();
    }

    if let Some(caps) = RE_UNIF.captures(s) {
      // FOR NOW UNWRAP
      let a: X = caps.get(1).unwrap().as_str().parse().ok().unwrap();
      let b: X = caps.get(3).unwrap().as_str().parse().ok().unwrap();
      let inc = !caps.get(2).unwrap().as_str().is_empty();
      Ok(Self::new(a, b, inc))
    } else {
      Err(Error::new(
        ErrorKind::Other,
        "metadata does not match!"
      ))
    }
  }
}

impl<X> fmt::Display for Range<X>
where X: Copy + fmt::Display + SampleUniform {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.inc {
      write!(f, "{}..={}", self.a, self.b)
    } else {
      write!(f, "{}..{}", self.a, self.b)
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  pub fn feedback() {
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
}
