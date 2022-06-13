use std::io::{BufRead, BufReader, Error};
use std::fs::File;
use std::path::Path;
use std::collections::{HashSet, HashMap};

pub const NLETS: usize = 5;
pub const NWORDS: usize = 2315;

#[derive(Debug)]
#[derive(Hash)]
#[derive(PartialEq, Eq, PartialOrd)]
#[derive(Clone, Copy)]
pub struct Word {
	pub data: [char; NLETS],
}

impl Word {
	pub fn from(s: &String) -> Result<Word, Error> {
		Ok(Word{data: s.chars()
						.collect::<Vec<char>>()
						.try_into()
						.expect("Expected string of length 5")})
	}

	pub fn to_string(&self) -> String {
		self.data.iter().collect()
	}
}

pub type WSet = HashSet<Word>;

pub fn get_words<P>(p: P) -> Result<WSet, Error> where P: AsRef<Path> {
	let file = File::open(p)?;
	let reader = BufReader::new(file);
	reader.lines()
		.filter_map(Result::ok)
		.map(|s| Word::from(&s))
		.collect()
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, PartialOrd, Clone, Copy)]
pub struct Feedback {
	// green + yellow bitsets
	g_bs: u8, 
	y_bs: u8,
}

impl Feedback {
	// answer, guess
	pub fn from(gw: Word, aw: Word) -> Self {
		let mut fb = Feedback{g_bs: 0, y_bs:0};
		for i in 0..NLETS {
			if aw.data[i] == gw.data[i] {
				fb.g_bs |= 1 << i;
			} else if aw.data.contains(&gw.data[i]) {
				fb.y_bs |= 1 << i;
			}
		};
		fb
	}

	pub fn from_str(s: &str) -> Self {
		let mut fb = Feedback{g_bs: 0, y_bs:0};
		for (i,c) in s.chars().take(NLETS).enumerate() {
			if c == 'G' {
				fb.g_bs |= 1 << i;
			} else if c == 'Y' {
				fb.y_bs |= 1 << i;
			}
		};
		fb
	}
	
	pub fn is_correct(&self) -> bool {
		self.g_bs == 31
	}
}

pub type FbMap<T> = HashMap<Feedback, T>; 

// decision tree
#[derive(Debug)]
pub enum DTree {
	Leaf,
	Node {
		// mean guesses to leaf
		eval: f64,
		// word
		word: Word,
		// children per unique feedback
		fbmap: FbMap<DTree>
	}
}

impl DTree {
	pub fn follow(self: &Self, fb: Feedback) -> Option<&DTree> {
		match self {
			DTree::Leaf => None,
			DTree::Node{eval, word, fbmap} => fbmap.get(&fb)
		}
	}

	pub fn get_eval(self: &Self) -> f64 {
		match self {
			DTree::Leaf => 0.0,
			DTree::Node{eval, word, fbmap} => *eval
		}
	}

	pub fn get_fbmap(self: &Self) -> Option<&FbMap<DTree>> {
		match self {
			DTree::Leaf => None,
			DTree::Node{eval, word, fbmap} => Some(fbmap)
		}
	}
}
