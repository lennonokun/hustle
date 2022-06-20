use std::io::{BufRead, BufReader, Error};
use std::fs::File;
use std::path::Path;
use rand::prelude::*;

use std::collections::{HashSet, HashMap};
use std::mem::MaybeUninit;

pub const NLETS: usize = 5;
pub const NGUESSES: usize = 6;
// pub const NAlPH: usize = 26;
pub const NWORDS: usize = 2309;
pub const MINWLEN: usize = 4;
pub const MAXWLEN: usize = 11;

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Word {
	pub data: [u8; MAXWLEN],
	pub wlen: u8,
}

impl Word {
	pub fn from(s: String) -> Option<Self> {
		let wlen = s.len() as u8;
		let mut data = [0u8; MAXWLEN];
		// ruins derived hash, eq, and ord
		// let mut data: [char; MAXWLEN] = unsafe {
			// MaybeUninit::uninit().assume_init()
		// };
		if s.len() > MAXWLEN {return None}
		for (i,c) in s.chars().enumerate() {
			data[i] = c as u8 - 'A' as u8;
		}
		Some(Word {data: data, wlen: wlen})
	}

	pub fn from_str(s: &str) -> Option<Self> {
		let wlen = s.len() as u8;
		let mut data = [0; MAXWLEN];
		// let mut data: [char; MAXWLEN] = unsafe {
			// MaybeUninit::uninit().assume_init()
		// };
		if s.len() > MAXWLEN {return None}
		for (i,c) in s.chars().enumerate() {
			data[i] = c as u8 - 'A' as u8;
		}
		Some(Word {data: data, wlen: wlen})
	}

	pub fn to_string(&self) -> String {
		self.data[0..self.wlen as usize]
			.iter().cloned()
			.map(|x| (x + 'A' as u8) as char)
			.collect()
	}
}

#[derive(Debug)]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Feedback {
	// green + yellow bitsets
	g_bs: u16, 
	y_bs: u16,
	wlen: u8,
}

impl Feedback {
	pub fn from(gw: Word, aw: Word) -> Option<Self> {
		if gw.wlen != aw.wlen {return None}
		let wlen = gw.wlen;
		let mut fb = Feedback{g_bs: 0, y_bs:0, wlen: wlen};
		// bitset for used chars
		let mut use_bs = 0u16;
		// first find greens
		for i in 0..wlen as usize {
			if aw.data[i] == gw.data[i] {
				fb.g_bs |= 1 << i;
				use_bs |= 1 << i;
			}
		}
		// then find yellows
		for i in 0..wlen as usize {
			if fb.g_bs & 1 << i == 0 {
				for j in 0..wlen as usize {
					if gw.data[i] == aw.data[j] && (use_bs & 1 << j == 0) {
						fb.y_bs |= 1 << i;
						use_bs |= 1 << j;
					}
				}
			}
		};
		Some(fb)
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
		};
		out
	}

	pub fn from_str(s: &str) -> Option<Self> {
		let wlen = s.len() as u8;
		if wlen > MAXWLEN as u8 {return None}
		let mut fb = Feedback{g_bs: 0, y_bs:0, wlen: wlen};
		for (i,c) in s.chars().take(wlen as usize).enumerate() {
			if c == 'G' {
				fb.g_bs |= 1 << i;
			} else if c == 'Y' {
				fb.y_bs |= 1 << i;
			}
		};
		Some(fb)
	}

	pub fn get_g(&self, i: u8) -> bool {
		return self.g_bs & 1 << i != 0;
	}
	
	pub fn get_y(&self, i: u8) -> bool {
		return self.y_bs & 1 << i != 0;
	}

	pub fn is_correct(&self) -> bool {
		self.g_bs == ((1 << self.wlen) - 1)
	}
}

pub struct WBank {
	pub data: Vec<Word>,
	pub wlen: u8,
}

impl WBank {
	pub fn from<P>(p: &P, wlen: u8) -> Result<Self, Error>
	where P: AsRef<Path> {
		let file = File::open(p)?;
		let reader = BufReader::new(file);
		let data = reader.lines()
			.filter_map(Result::ok)
			.filter(|s| s.len() == wlen.into())
			.filter_map(Word::from)
			.collect::<Vec<Word>>();
		Ok(WBank{data: data, wlen: wlen})
	}

	pub fn new() -> Self {
		WBank {data: Vec::new(), wlen: 0}
	}

	pub fn contains(&self, w: Word) -> bool {
		self.data.contains(&w)
	}

	pub fn pick(&self, rng: &mut ThreadRng, n: usize) -> Vec<Word> {
		self.data
			.choose_multiple(&mut rand::thread_rng(), n)
			.cloned()
			.collect()
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
