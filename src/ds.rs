use std::io::{BufRead, BufReader, Error};
use std::fs::File;
use std::path::Path;
use std::collections::{HashSet, HashMap};
use std::mem::MaybeUninit;

pub const NLETS: usize = 5;
// pub const NAlPH: usize = 26;
pub const NWORDS: usize = 2309;

#[derive(Debug)]
#[derive(Hash)]
#[derive(PartialEq, Eq, PartialOrd)]
#[derive(Clone, Copy)]
pub struct Word {
	pub data: [char; NLETS],
}

impl Word {
	pub fn from(s: &String) -> Option<Word> {
		if s.len() != NLETS {return None}
		let data = MaybeUninit::<[char; 5]>::uninit();
		let mut data = unsafe {data.assume_init()};
		for (i,c) in &mut s.chars().enumerate() {
			data[i] = c;
		}
		Some(Word{data: data})
	}

	pub fn from_str(s: &str) -> Option<Word> {
		if s.len() != NLETS {return None}
		let data = MaybeUninit::<[char; 5]>::uninit();
		let mut data2 = unsafe {data.assume_init()};
		for (i,c) in &mut s.chars().enumerate() {
			data2[i] = c;
		}
		Some(Word{data: data2})
	}

	pub fn to_string(&self) -> String {
		self.data.iter().collect()
	}
}

pub type WSet = HashSet<Word>;
pub type WArr = [Word; NWORDS];

pub fn get_words<P>(p: P) -> Result<WSet, Error> where P: AsRef<Path> {
	let file = File::open(p)?;
	let reader = BufReader::new(file);
	Ok(reader.lines()
		.filter_map(Result::ok)
		.filter_map(|s| Word::from(&s))
		.collect())
}

pub fn get_awarr<P>(p: P) -> Result<WArr, Error> where P: AsRef<Path> {
	let file = File::open(p)?;
	let reader = BufReader::new(file);
	Ok(reader.lines()
		.filter_map(Result::ok)
		.filter_map(|s| Word::from(&s))
		.collect::<Vec<Word>>()
		.try_into()
		.expect("expected results of length NWORDS"))
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Feedback {
	// green + yellow bitsets
	g_bs: u8, 
	y_bs: u8,
}

impl Feedback {
	// answer, guess
	pub fn from(gw: Word, aw: Word) -> Self {
		let mut fb = Feedback{g_bs: 0, y_bs:0};
		// bitset for used chars
		let mut use_bs = 0u8;
		// first find greens
		for i in 0..NLETS {
			if aw.data[i] == gw.data[i] {
				fb.g_bs |= 1 << i;
				use_bs |= 1 << i;
			}
		}
		// then find yellows
		for i in 0..NLETS {
			if fb.g_bs & 1 << i == 0 {
				for j in 0..NLETS {
					if gw.data[i] == aw.data[j] && (use_bs & 1 << j == 0) {
						fb.y_bs |= 1 << i;
						use_bs |= 1 << j;
					}
				}
			}
		};
		fb
	}

	pub fn to_string(&self) -> String {
		let mut out = String::new();
		for i in 0..NLETS {
			if self.g_bs & 1 << i != 0 {
				out.push('G');
			} else if self.y_bs & 1 << i != 0 {
				out.push('Y');
			} else {
				out.push('B');
			}
		};
		out
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

	pub fn get_g(&self, i: usize) -> bool {
		return self.g_bs & 1 << i != 0;
	}
	
	pub fn get_y(&self, i: usize) -> bool {
		return self.y_bs & 1 << i != 0;
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

	pub fn pprint(self: &Self, indent: &String, n: i32) {
		match self {
			DTree::Leaf => {}
			DTree::Node{eval, word, fbmap} => {
				println!("{}{}, {}", indent, word.to_string(), eval);
				let mut indent2 = indent.clone();
				indent2.push(' ');
				let mut items : Vec<(&Feedback, &DTree)> =
					fbmap.iter().collect();
				items.sort_by(|(_, dt1), (_, dt2)|
					dt1.get_eval().partial_cmp(&dt2.get_eval()).unwrap());
				for (fb, dt) in items {
					println!("{}{}{}", indent2, fb.to_string(), n);
					dt.pprint(&indent2,n+1);
				}
			}
		}
	}
}
