use std::fs::File;
use std::path::Path;
use std::io::{self, Write, BufRead, BufReader};

use super::feedback::{Feedback, FbMap};
use super::word::Word;

#[derive(Debug)]
struct NodeStruct {
  // total leaf depth
  pub tot: u32,
  // word
  pub word: Word,
  // children per unique feedback
  pub fbmap: FbMap<DTree>,
}

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

impl From<NodeStruct> for DTree {
  fn from(item: NodeStruct) -> Self {
    DTree::Node {
      tot: item.tot,
      word: item.word,
      fbmap: item.fbmap,
    }
  }
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

  pub fn get_alen(&self) -> u32 {
    match self {
      DTree::Leaf => 1,
      DTree::Node {
        tot,
        word: _,
        fbmap,
      } => {
        fbmap.iter()
          .map(|(fb, dt)| dt.get_alen())
          .sum()
      }
    }
  }

  pub fn get_eval(&self) -> f64 {
    self.get_tot() as f64 / self.get_alen() as f64
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

  pub fn load(p: &Path) -> Option<Self> {
    let f = File::open(p).ok()?;
    let br = BufReader::new(f);

    let mut dt_stack = Vec::<NodeStruct>::new();
    let mut fb_stack = Vec::<Feedback>::new();

    let mut mode = true;
    for line in br.lines().filter_map(|s| s.ok()) {
      // compact dtree to indentation level
      let indent = line.chars().enumerate().find(|(_i, c)| *c != ' ')?.0;
      while indent < dt_stack.len() {
        let dt = dt_stack.pop()?;
        let fb = fb_stack.pop()?;
        let mut last_dt = dt_stack.last_mut()?;
        last_dt.fbmap.insert(fb, dt.into());
      }

      let line = line.trim();
      if mode {
        // read word+tot and push empty dtree
        let mut split = line.split(",");
        let word = Word::from_str(split.next()?)?;
        let tot = split.next()?.trim().parse::<u32>().ok()?;
        dt_stack.push(NodeStruct {tot, word, fbmap: FbMap::new()});
        mode = false;
      } else {
        // read feedback
        let fb = Feedback::from_str(&line[0..line.len()-1])?;
        if fb.is_correct() {
          // add leaf to last dtree
          let mut last_dt = dt_stack.last_mut()?;
          last_dt.fbmap.insert(fb, DTree::Leaf);
        } else {
          // push feedback
          fb_stack.push(fb);
          mode = true;
        }
      }
    }

    // compact into one dtree and return
    while dt_stack.len() > 1 {
      let dt = dt_stack.pop()?;
      let fb = fb_stack.pop()?;
      let mut last_dt = dt_stack.last_mut()?;
      last_dt.fbmap.insert(fb, dt.into());
    }
    Some(dt_stack.pop()?.into())
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

#[cfg(test)]
mod test {
  use std::fs::{File, remove_file};
  use std::path::Path;
  use lazy_static::lazy_static;

  use super::*;

  macro_rules! dtree {
    ($word: expr, $tot: expr, $($fb: expr, $dt: expr,)*) => {{
      let mut fbmap = FbMap::<DTree>::new();
      $( fbmap.insert(Feedback::from_str($fb).unwrap(), $dt); )*
      DTree::Node{word: Word::from_str($word).unwrap(), tot: $tot, fbmap}
    }}
  }

  lazy_static! {
    static ref TEST_DTS: Vec<DTree> = vec![dtree!(
      "salet", 5,
      "bbbbb", dtree!("courd", 3,
                      "ggggg", DTree::Leaf,
                      "bbbbb", dtree!("nymph", 1, "ggggg", DTree::Leaf,),),
      "gbbbb", dtree!("mucho", 1, "ggggg", DTree::Leaf,),
      "bgbbb", dtree!("corny", 1, "ggggg", DTree::Leaf,),
      "bbgbb", dtree!("fugly", 1, "ggggg", DTree::Leaf,),
      "bbbgb", dtree!("rownd", 1, "ggggg", DTree::Leaf,),
      "bbbbg", dtree!("groin", 1, "ggggg", DTree::Leaf,),
    ), dtree!(
     "reast", 1,
     "ggggg", DTree::Leaf,
    ), dtree!(
     "lodge", 5,
     "bbbbb", dtree!(
       "saint", 4,
       "bbbbb", dtree!(
         "chump", 3,
         "bbybb", dtree!(
           "furry", 2,
           "ggbbg", dtree!("fuzzy", 1, "ggggg", DTree::Leaf,),
         ),
       ),
     ),
   ),];
  }

  #[test]
  fn save_load() {
    for test_dt in &*TEST_DTS {
      let mut f = File::create("blah").unwrap();
      test_dt.pprint(&mut f, &("".to_owned()), 0);
      let dt = DTree::load(Path::new("blah")).unwrap();
      remove_file("blah").unwrap();
      assert_eq!(*test_dt, dt);
    }
  }
}
