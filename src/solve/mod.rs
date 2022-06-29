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

struct SolveData{
	dt: Option<DTree>,
	beta: u32
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
