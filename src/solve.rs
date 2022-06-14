use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use rayon::prelude::*;

use crate::ds::{Word, Feedback, DTree, WSet, FbMap, NLETS};
use crate::analysis::{HData, HRec};

const NTOPS: usize = 20;
const ENDGCUTOFF: usize = 20;

// TODO:
// better understand borrowing
// are inf counts useful?
// keep track of n for hdata?

// get feedback partitions
pub fn fb_partition(gw: Word, aws: &WSet) -> FbMap<WSet> {
	let mut map = HashMap::new();
	for aw in aws {
		let fb = Feedback::from(gw, *aw);
		let set : &mut WSet =
			map.entry(fb).or_insert(HashSet::new());
		set.insert(*aw);
	};
	map
}

// get feedback partition counts
pub fn fb_counts(gw: Word, aws: &WSet) -> FbMap<i32> {
	let mut map = HashMap::new();
	for aw in aws {
		let fb = Feedback::from(gw, *aw);
		*map.entry(fb).or_insert(0) += 1
	};
	map
}

// apply precalculated heuristic to partition sizes (lower is better)
pub fn heuristic(gw: Word, aws: &WSet, hd: &HData) -> f64 {
	fb_counts(gw, aws).iter()
		.map(|(_, n)| hd.get_approx(*n as usize))
		.sum()
}

// get top n words based off of heuristic
pub fn top_words(gws: &WSet, aws: &WSet, hd: &HData, n: usize)
	-> Vec<Word> {
	let mut tups : Vec<(Word, f64)> = gws.iter()
		.map(|gw| (*gw, heuristic(*gw, aws, hd)))
		.collect();
	tups.sort_by(|(_, f1), (_, f2)| f1.partial_cmp(f2).unwrap());
	tups.iter()
		.map(|(gw, _)| *gw)
		.take(n)
		.collect()
}

// NOT WORTH
pub fn common_letters(w1: Word, w2: Word) -> i32 {
	let mut out = 0;
	for i in 0..NLETS {
		if w1.data[i] == w2.data[i] {
			out += 1;
		}
	}
	out
}

pub fn reduce_words(gw: Word, gws: &WSet) -> WSet {
	gws.iter()
		// why double?
		.filter(|gw2| common_letters(gw, **gw2) <= 1)
		.copied()
		.collect()
}

// get upper bound for minimum mean guesses at state given guess
pub fn solve_given(gw: Word, gws: &WSet, aws: &WSet, n: i32,
							 hd: &HData, hrm: &Mutex<HRec>) -> Option<DTree> {
	// unnecessary unless user is dumb
	let alen = aws.len();
	if alen == 1 && gw == *aws.iter().next().unwrap() {
		return Some(DTree::Leaf);
	} else if n == 0 || (n == 1 && alen > 20) {
		return None
	}

	let mut eval = 1.0;
	let mut fbmap = FbMap::new();
	for (fb, set) in fb_partition(gw, aws) {
		if !fb.is_correct() {
			let dt2 = solve_state(&gws, &set, n-1, hd, hrm);
			// let dt2 = if alen > ENDGCUTOFF {
				// solve_state(&reduce_words(gw, &gws), &set, n-1, hd)
			// } else {
				// solve_state(&gws, &set, n-1, hd)
			// };
			match dt2 {
				None => {
					return None;
				} Some(dt2) => {
					eval += (set.len() as f64/alen as f64) * dt2.get_eval();
					fbmap.insert(fb, dt2);
				}
			}
		} 
	}

	return Some(DTree::Node{
		eval:eval, word:gw, fbmap:fbmap
	});
}

struct SolveData {
	dt: Option<DTree>,
	eval: f64,
	stop: bool,
}

// get upper bound for mean guesses at state
pub fn solve_state(gws: &WSet, aws: &WSet, n: i32,
							 hd: &HData, hrm: &Mutex<HRec>) -> Option<DTree> {
	// worth?
	let alen = aws.len();
	if alen == 1 {
		// 100% chance for one guess
		hrm.lock().unwrap().record(1, 1.0);
		return Some(DTree::Node{
			eval: 1.0, 
			word: *aws.iter().next().unwrap(),
			fbmap: [(Feedback::from_str("GGGGG"), DTree::Leaf)].into()
		});
	}

	if n == 0 {
		hrm.lock().unwrap().record_inf(n as usize);
		return None
	}

	// todo update comments for no above?
	let sd = Mutex::new(SolveData{
		dt: Some(DTree::Leaf),
		eval: f64::INFINITY,
		stop: false,
	});

	// in "endgame", check if guessing a possible
	// answer guarantees correct next guess (score < 2)
	// (so far alen = 15 is max i've found where this is possible)
	if alen <= ENDGCUTOFF {
		aws.into_par_iter().for_each(|aw| {
			// check stop
			if sd.lock().unwrap().stop {return}
			match solve_given(*aw, gws, aws, n, hd, hrm) {
				None => {},
				Some(dt2) => {
					// check stop
					let mut sd2 = sd.lock().unwrap();
					let eval2 = dt2.get_eval();
					if sd2.stop {
						return;
					} else if eval2 < 1.999 {
						// hd.record(alen, eval2); 
						sd2.dt = Some(dt2);
						sd2.eval = eval2;
						sd2.stop = true;
					} else if eval2 < sd2.eval {
						sd2.dt = Some(dt2);
						sd2.eval = eval2;
					}
				}
			}
		});
	}
	// dont bother checking other words if a 2 was found
	if sd.lock().unwrap().eval < 2.001 {
		// cringe?
		hrm.lock().unwrap().record(alen, sd.lock().unwrap().eval); 
		return sd.into_inner().unwrap().dt;
	}

	// search top heuristic words and stop
	// on guaranteed next guess (score = 2)
	top_words(gws, aws, hd, NTOPS)
		.into_par_iter()
		.for_each(|gw| {
			if sd.lock().unwrap().stop {return}
			let dt2 = solve_given(gw, gws, aws, n, hd, hrm);
			match dt2 {
				None => {},
				Some(dt2) => {
					let mut sd2 = sd.lock().unwrap();
					let eval2 = dt2.get_eval(); 
					if sd2.stop {
						return;
					} else if eval2 < 2.001 {
						sd2.dt = Some(dt2);
						sd2.eval = eval2;
						sd2.stop = true;
					} else if eval2 < sd2.eval {
						sd2.dt = Some(dt2);
						sd2.eval = eval2;
					}
				}
			};
		});

	let sd2 = sd.into_inner().unwrap();
	if sd2.eval == f64::INFINITY {
		// todo why inf?
		hrm.lock().unwrap().record_inf(n as usize);
		return None;
	} else {
		hrm.lock().unwrap().record(alen, sd2.eval);
		return sd2.dt;
	}
}
