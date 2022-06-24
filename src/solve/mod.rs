use std::sync::Mutex;
use rayon::prelude::*;

use crate::ds::*;

pub mod analysis;
use crate::solve::analysis::HData;
pub mod util;
use crate::solve::util::*;

// number of top words to try
const NTOPS: usize = 1;
// number of remaining words makes it "endgame"
const ENDGCUTOFF: usize = 15;

struct GivenData {
	fbmap: FbMap<DTree>,
	tot: u32,
	stop: bool,
}

// get upper bound for minimum mean guesses at state given guess
pub fn solve_given(gw: Word, gwb: &WBank, awb: &WBank,
									 n: u32, hd: &HData) -> Option<DTree> { 
	let alen = awb.data.len();

	if alen == 1 && gw == *awb.data.iter().next().unwrap() {
		// leaf if guessed
		return Some(DTree::Leaf);
	} else if n == 0 || (n == 1 && alen > 20) {
		// impossible guesses
		return None
	}

	let gd = Mutex::new(GivenData{
		fbmap: FbMap::new(),
		tot: alen as u32,
		stop: false,
	});

	fb_partition(gw, awb).into_par_iter().for_each(|(fb, wb)| {
		if gd.lock().unwrap().stop {return}
		if fb.is_correct() {
			let mut gd2 = gd.lock().unwrap();
			gd2.fbmap.insert(Feedback::from_str("GGGGG").unwrap(),
											 DTree::Leaf);
		} else {
			// worth?
			let dt2 = if alen > ENDGCUTOFF {
				solve_state(&reduce_words(gw, &gwb), &wb, n-1, hd)
			} else {
				solve_state(&gwb, &wb, n-1, hd)
			};
			if gd.lock().unwrap().stop {return}
			let mut gd2 = gd.lock().unwrap();
			match dt2 {
				None => {
					// give up if inf
					gd2.stop = true;
				} Some(dt2) => {
					gd2.tot += dt2.get_tot();
					gd2.fbmap.insert(fb, dt2);
				}
			}
		} 
	});

	// construct dtree
	let gd2 = gd.into_inner().unwrap();
	if gd2.stop {return None}
	return Some(DTree::Node{
		tot: gd2.tot, word: gw, fbmap: gd2.fbmap
	});
}

struct SolveData {
	dt: Option<DTree>,
	tot: u32,
	stop: bool,
}

// get upper bound for mean guesses at state
pub fn solve_state(gwb: &WBank, awb: &WBank,
									 n: u32, hd: &HData) -> Option<DTree> {
	let alen = awb.data.len();

	// inf when ran out of turns
	if n == 0 {
		// hd.hrm.lock().unwrap().record_inf(n as usize);
		return None
	}

	// one answer -> guess it
	if alen == 1 {
		hd.hrm.lock().unwrap().record(alen as usize, 1);
		return Some(DTree::Node{
			tot: 1, 
			word: *awb.data.iter().next().unwrap(),
			fbmap: [(Feedback::from_str("GGGGG").unwrap(),
							 DTree::Leaf)].into()
		});
	}

	let sd = Mutex::new(SolveData{
		dt: Some(DTree::Leaf),
		tot: u32::MAX,
		stop: false,
	});

	// in "endgame", check if guessing a possible
	// answer guarantees correct next guess (score < 2)
	// (so far alen = 15 is max i've found where this is possible)
	if alen <= ENDGCUTOFF {
		(&awb.data).into_par_iter().for_each(|aw| {
			// check stop
			if sd.lock().unwrap().stop {return}
			match solve_given(*aw, gwb, awb, n, hd) {
				None => {},
				Some(dt2) => {
					// check stop
					let mut sd2 = sd.lock().unwrap();
					let tot2 = dt2.get_tot();
					if sd2.stop {
						return;
					} else if tot2 < alen as u32 {
						sd2.dt = Some(dt2);
						sd2.tot = tot2;
						sd2.stop = true;
					} else if tot2 < sd2.tot {
						sd2.dt = Some(dt2);
						sd2.tot = tot2;
					}
				}
			}
		});
	}
	// dont bother checking other words if a 2 was found
	if sd.lock().unwrap().tot == alen as u32 {
		hd.hrm.lock().unwrap().record(alen as usize,
																	sd.lock().unwrap().tot); 
		return sd.into_inner().unwrap().dt;
	}

	// search top heuristic words and stop
	// on guaranteed next guess (score = 2)
	top_words(gwb, awb, hd, NTOPS)
		.into_par_iter()
		.for_each(|gw| {
			if sd.lock().unwrap().stop {return}
			let dt2 = solve_given(gw, gwb, awb, n, hd);
			match dt2 {
				None => {},
				Some(dt2) => {
					let mut sd2 = sd.lock().unwrap();
					let tot2 = dt2.get_tot(); 
					if sd2.stop {
						return;
					} else if tot2 < alen as u32 {
						// can't do better
						sd2.dt = Some(dt2);
						sd2.tot = tot2;
						sd2.stop = true;
					} else if tot2 < sd2.tot {
						// update best
						sd2.dt = Some(dt2);
						sd2.tot = tot2;
					}
				}
			};
		});

	let sd2 = sd.into_inner().unwrap();
	if sd2.tot == u32::MAX {
		// todo why inf?
		return None;
	} else {
		hd.hrm.lock().unwrap().record(alen as usize, sd2.tot);
		return sd2.dt;
	}
}
