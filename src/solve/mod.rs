use std::sync::Mutex;
use rayon::prelude::*;
use rand::prelude::*;

use crate::ds::*;

pub mod analysis;
use crate::solve::analysis::HData;
pub mod util;
use crate::solve::util::*;


#[derive(Clone, Copy)]
pub struct Config {
	// number of top words to try
	pub ntops: u32,
	// number of remaining words makes it "endgame"
	pub endgcutoff: u32,
	// whether or not to record
	pub record: bool
}

struct SolveData{
	dt: Option<DTree>,
	beta: u32
}

#[derive(Clone)]
pub struct State<'a> {
	pub gwb: &'a WBank,
	pub awb: &'a WBank,
	pub hd: &'a HData,
	pub cfg: Config,
	pub turns: u32,
	pub beta: u32
}

impl<'a> State<'a> {
	pub fn new(gwb: &'a WBank, awb: &'a WBank,
						 hd: &'a HData, cfg: Config) -> Self {
		State::<'a> {gwb, awb, hd, cfg, turns: 6, beta: u32::MAX}
	}

	// pub fn fb_partition(&self, gw: Word) -> FbMap<State> {
		// let mut map = FbMap::new();
		// for aw in &self.awb.data {
			// let fb = Feedback::from(gw, *aw).unwrap();
			// map.entry(fb).or_insert_with(|| {
				// let mut s2 = self.clone();
				// // let empty_wb = WBank::new2(self.awb.wlen);
				// // s_empty.awb = &empty_wb;
				// s2.awb = &WBank::new2(self.awb.wlen);
				// s2.turns = self.turns - 1;
				// s2
			// }).awb.data.push(*aw);
		// };
		// map
	// }
	
	pub fn solve_given(&self, gw: Word) -> Option<DTree> {
		let alen = self.awb.data.len();
		if alen == 1 && gw == *self.awb.data.get(0).unwrap() {
			// leaf if guessed
			return Some(DTree::Leaf);
		} else if self.turns == 0 || (self.turns == 1 && alen > 20) {
			// impossible guesses
			return None;
		}

		let mut tot = alen as u32;
		let mut fbm = FbMap::new();

		for (fb, wb) in fb_partition(gw, self.awb).iter() {
			if fb.is_correct() {
				fbm.insert(*fb, DTree::Leaf);
			} else {
				let mut s = self.clone();
				s.awb = &wb;
				s.turns = self.turns - 1;
				let dt = s.solve_state();
				match dt {
					None => return None,
					Some(dt) => {
						tot += dt.get_tot();
						fbm.insert(*fb, dt);
						if tot >= self.beta {return None}
					}
				}
			}

		}

		Some(DTree::Node{tot, word: gw, fbmap: fbm})
	}

	pub fn solve_state(&self) -> Option<DTree> {
		let alen = self.awb.data.len();

		// no more turns
		if self.turns == 0 {
			return None;
		// one answer -> guess it
		} else if alen == 1 {
			if self.cfg.record {self.hd.hrm.lock().unwrap().record(alen, 1)}
			return Some(DTree::Node{
				tot: 1,
				word: *self.awb.data.get(0).unwrap(),
				fbmap: [(Feedback::from_str("GGGGG").unwrap(),
								DTree::Leaf)].into()
			});
		}

		// check if endgame guess is viable
		if alen <= self.cfg.endgcutoff as usize {
			for aw in self.awb.data.iter() {
				if fb_counts(*aw, self.awb).values().all(|c| *c==1) {
					if self.cfg.record {
						self.hd.hrm.lock().unwrap().record(alen, 2*alen as u32 - 1);
					}
					return self.solve_given(*aw);
				}
			}
		}

		// finally, check top words
		let sd = Mutex::new(SolveData{
			dt: None, beta: self.beta
		});

		top_words(self.gwb, self.awb, self.hd, self.cfg.ntops as usize)
			.into_par_iter()
			.for_each(|gw| {
				if sd.lock().unwrap().beta <= 2 * alen as u32 {return}
				let mut s = self.clone();
				s.beta = sd.lock().unwrap().beta;
				let dt2 = s.solve_given(gw);
				let mut sd2 = sd.lock().unwrap();
				if let Some(dt2) = dt2 {
					if dt2.get_tot() < sd2.beta {
						sd2.beta = dt2.get_tot();
						sd2.dt = Some(dt2);
					}
				}
			});

		let dt = sd.into_inner().unwrap().dt;
		if self.cfg.record {
			if let Some(ref dt) = dt {
				self.hd.hrm.lock().unwrap().record(alen, dt.get_tot());
			}
		}
		dt
	}
}

pub fn solve_given(gw: Word, gwb: &WBank, awb: &WBank, n: u32,
									 hd: &HData, cfg: Config, beta: u32)
									 -> Option<DTree> {
	let alen = awb.data.len();
	if alen == 1 && gw == *awb.data.get(0).unwrap() {
		// leaf if guessed
		return Some(DTree::Leaf);
	} else if n == 0 || (n == 1 && alen > 20) {
		// impossible guesses
		return None;
	}

	let mut tot = alen as u32;
	let mut fbm = FbMap::new();

	for (fb, wb) in fb_partition(gw, awb).iter() {
		if fb.is_correct() {
			fbm.insert(*fb, DTree::Leaf);
		} else {
			let dt = solve_state(gwb, &wb, n-1, hd, cfg, beta);
			match dt {
				None => return None,
				Some(dt) => {
					tot += dt.get_tot();
					fbm.insert(*fb, dt);
					if tot >= beta {return None}
				}
			}
		}
	}

	Some(DTree::Node{tot, word: gw, fbmap: fbm})
}

pub fn solve_state(gwb: &WBank, awb: &WBank, n: u32,
										hd: &HData, cfg: Config, beta: u32)
										-> Option<DTree> {
	let alen = awb.data.len();

	// no more turns
	if n == 0 {
		return None;
	// one answer -> guess it
	} else if alen == 1 {
		if cfg.record {hd.hrm.lock().unwrap().record(alen, 1)}
		return Some(DTree::Node{
			tot: 1,
			word: *awb.data.get(0).unwrap(),
			fbmap: [(Feedback::from_str("GGGGG").unwrap(),
							 DTree::Leaf)].into()
		});
	}

	// check if endgame guess is viable
	if alen <= cfg.endgcutoff as usize {
		for aw in awb.data.iter() {
			if fb_counts(*aw, awb).values().all(|c| *c==1) {
				if cfg.record {
					hd.hrm.lock().unwrap().record(alen, 2*alen as u32 - 1);
				}
				return solve_given(*aw, gwb, awb, n, hd, cfg, beta);
			}
		}
	}
	
	// finally, check top words
	let sd = Mutex::new(SolveData{
		dt: None, beta
	});

	top_words(gwb, awb, hd, cfg.ntops as usize)
		.into_par_iter()
		.for_each(|gw| {
			if sd.lock().unwrap().beta <= 2 * alen as u32 {return}
			let dt2 = solve_given(gw, gwb, awb, n, hd, cfg,
														 sd.lock().unwrap().beta);
			let mut sd2 = sd.lock().unwrap();
			if let Some(dt2) = dt2 {
				if dt2.get_tot() < sd2.beta {
					sd2.beta = dt2.get_tot();
					sd2.dt = Some(dt2);
				}
			}
		});

	let dt = sd.into_inner().unwrap().dt;
	if cfg.record {
		if let Some(ref dt) = dt {
			hd.hrm.lock().unwrap().record(alen, dt.get_tot());
		}
	}
	dt
}
