use std::io::Write;

use super::feedback::{Feedback, FbMap};
use super::word::Word;

/// decision tree
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
        items.sort_by_key(|(fb, _dt)| fb.to_id());
        for (fb, dt) in items {
          writeln!(out, "{}{}{}", indent2, fb.to_string(), n);
          dt.pprint(out, &indent2, n + 1);
        }
      }
    }
  }
}

