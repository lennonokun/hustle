use std::io::{Result, BufRead, BufReader, Error, Write};
use std::fs::File;
use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use rand::prelude::*;

pub const NLETS: usize = 5;
pub const NGUESSES: usize = 6;
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
		if s.len() > MAXWLEN {return None}
		for (i,c) in s.to_ascii_uppercase().chars().enumerate() {
			data[i] = c as u8 - b'A';
		}
		Some(Word{data, wlen})
	}

	pub fn from_str(s: &str) -> Option<Self> {
		let wlen = s.len() as u8;
		let mut data = [0; MAXWLEN];
		if s.len() > MAXWLEN {return None}
		for (i,c) in s.to_ascii_uppercase().chars().enumerate() {
			data[i] = c as u8 - b'A';
		}
		Some(Word{data, wlen})
	}

	pub fn to_string(&self) -> String {
		self.data[0..self.wlen as usize]
			.iter().cloned()
			.map(|x| (x + b'A') as char)
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
		let mut fb = Feedback{g_bs: 0, y_bs:0, wlen};
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
						break;
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
		let mut fb = Feedback{g_bs: 0, y_bs:0, wlen};
		for (i,c) in s.to_ascii_uppercase().chars()
			.take(wlen as usize).enumerate() {
			if c == 'G' {
				fb.g_bs |= 1 << i;
			} else if c == 'Y' {
				fb.y_bs |= 1 << i;
			}
		};
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

// pub struct WBank {
	// pub gws: Vec<Word>,
	// pub aws: Arc<Vec<Word>,
	// pub wlen: u8
// }

#[derive(Debug, Clone)]
pub struct WBank {
	pub data: Vec<Word>,
	pub wlen: u8,
}

impl WBank {
	pub fn from<P>(p: &P, wlen: u8) -> Result<Self>
	where P: AsRef<Path> {
		let file = File::open(p)?;
		let reader = BufReader::new(file);
		let data = reader.lines()
			.filter_map(Result::ok)
			.filter(|s| s.len() == wlen.into())
			.filter_map(Word::from)
			.collect::<Vec<Word>>();
		Ok(WBank{data, wlen})
	}

	pub fn from2<P>(p: P, wlen: u8) -> Result<(Self, Self)>
	where P: AsRef<Path> {
		let file = File::open(p)?;
		let reader = BufReader::new(file);
		let mut gdata = Vec::<Word>::new();
		let mut adata = Vec::<Word>::new();
		for line in reader.lines().skip(1).flatten() {
			// parse line
			let vec: Vec<&str> = line.split(',').collect();
			if vec[2].parse::<u8>().unwrap() != wlen {continue}
			// push to both if answer word, but only guess if guess word
			let w = Word::from_str(vec[0]).unwrap();
			if vec[1] == "A" {adata.push(w)}
			gdata.push(w);
		}

		Ok((WBank {data: gdata, wlen}, WBank {data: adata, wlen}))
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

	pub fn to_string(&self) -> String {
		let s = self.data.iter()
			.map(|w| w.to_string())
			.collect::<Vec<String>>().join(" ");
		format!("[{s}]")
	}
}

pub type FbMap<T> = HashMap<Feedback, T>; 

// decision tree
#[derive(Debug)]
pub enum DTree {
	Leaf,
	Node {
		// total leaf depth
		tot: u32,
		// word
		word: Word,
		// children per unique feedback
		fbmap: FbMap<DTree>
	}
}

impl DTree {
	pub fn follow(&self, fb: Feedback) -> Option<&DTree> {
		match self {
			DTree::Leaf => None,
			DTree::Node{tot, word, fbmap} => fbmap.get(&fb)
		}
	}

	pub fn get_tot(&self) -> u32 {
		match self {
			DTree::Leaf => 0,
			DTree::Node{tot, word, fbmap} => *tot
		}
	}

	pub fn get_fbmap(&self) -> Option<&FbMap<DTree>> {
		match self {
			DTree::Leaf => None,
			DTree::Node{tot, word, fbmap} => Some(fbmap)
		}
	}

	pub fn pprint<W>(&self, out: &mut W, indent: &String, n: u32)
		where W: Write {
		match self {
			DTree::Leaf => {}
			DTree::Node{tot, word, fbmap} => {
				writeln!(out, "{}{}, {}", indent, word.to_string(), tot);
				let mut indent2 = indent.clone();
				indent2.push(' ');
				let mut items : Vec<(&Feedback, &DTree)> =
					fbmap.iter().collect();
				items.sort_by_key(|(_fb, dt)| -(dt.get_tot() as i32));
				for (fb, dt) in items {
					writeln!(out, "{}{}{}", indent2, fb.to_string(), n);
					dt.pprint(out, &indent2,n+1);
				}
			}
		}
	}
}
