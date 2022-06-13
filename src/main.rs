use std::collections::{HashMap, HashSet};

mod ds;
use crate::ds::{Word, Feedback, DTree, WSet, FbMap, get_words};

mod analysis;
use crate::analysis::{HTable, read_heuristic};

// TODO:
// multithread
// better understand borrowing
// heuristic is worse now for bigger partitions bc i dont sample enough

// get feedback partitions
fn fb_partition(gw: Word, aws: &WSet) -> FbMap<WSet> {
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
fn fb_counts(gw: Word, aws: &WSet) -> FbMap<i32> {
	let mut map = HashMap::new();
	for aw in aws {
		let fb = Feedback::from(gw, *aw);
		*map.entry(fb).or_insert(0) += 1
	};
	map
}

// apply precalculated heuristic to partition sizes (lower is better)
fn heuristic(gw: Word, aws: &WSet, ht: &HTable) -> f64 {
	fb_counts(gw, aws).iter()
		.map(|(_, n)| ht[*n as usize])
		.sum()
}

// get top n words based off of heuristic
fn top_words(gws: &WSet, aws: &WSet, ht: &HTable, n: usize)
	-> Vec<Word> {
	let mut tups : Vec<(Word, f64)> = gws.iter()
		.map(|gw| (*gw, heuristic(*gw, aws, ht)))
		.collect();
	tups.sort_by(|(_, f1), (_, f2)| f1.partial_cmp(f2).unwrap());
	tups.iter()
		.map(|(gw, _)| *gw)
		.take(n)
		.collect()
}

// NOT WORTH
// fn common_letters(w1: Word, w2: Word) -> i32 {
	// let mut out = 0;
	// for i in 0..NLETS {
		// if w1.data[i] == w2.data[i] {
			// out += 1;
		// }
	// }
	// out
// }
// 
// fn reduce_words(gw: Word, gws: &WSet, aws: &WSet) -> WSet {
	// gws.iter()
		// // why double?
		// .filter(|gw2| common_letters(gw, **gw2) <= 1 || aws.contains(gw2))
		// .copied()
		// .collect()
// }

// get upper bound for minimum mean guesses at state given guess
fn solve_given(gw: Word, gws: &WSet, aws: &WSet,
							n: i32, ht: &HTable) -> Option<DTree> {
	// unnecessary unless user is dumb
	// if aws.len() == 1 && gw == aws.iter().next().unwrap() {
		// return 0.0;
	// }
	// todo if n == 1 && aws.len() > 20?
	if n == 0 {return None}

	let mut eval = 1.0;
	let mut fbmap = FbMap::new();
	for (fb, set) in fb_partition(gw, aws) {
		if !fb.is_correct() {
			let dt2 = solve_state(gws, &set, n-1, ht);
			match dt2 {
				None => return None,
				Some(dt2) => {
					eval += (set.len() as f64/aws.len() as f64) * dt2.get_eval();
					fbmap.insert(fb, dt2);
				}
			}
		} 
	}

	return Some(DTree::Node{
		eval:eval, word:gw, fbmap:fbmap
	});
}

// get upper bound for mean guesses at state
fn solve_state(gws: &WSet, aws: &WSet, n: i32,
							ht: &HTable) -> Option<DTree> {
	// worth?
	// if aws.len() == 1 {
		// // 100% chance for one guess
		// return Some(DTree::Node{
			// eval: 1.0, 
			// word: *aws.iter().next().unwrap(),
			// fbmap: [(fb_solved, DTree::Leaf)].into()
		// });
		// return 
	// } else if aws.len() == 2 {
		// // 50% chance for one guess
		// return 1.5;
		// return Some(DTree::Node{
			// eval: 1.5, 
			// word: *aws.iter().next().unwrap(),
			// fbmap: [(fb_solved, DTree::Leaf)].into()
		// });

	// todo update comments for no above?
	let mut dt = Some(DTree::Leaf);
	let mut eval = f64::INFINITY;

	// in "endgame", check if guessing a possible
	// answer guarantees correct next guess (score < 2)
	// (so far aws.len() = 14 is max i've found where this is possible)
	if aws.len() < 10 {
		for aw in aws {
			match solve_given(*aw, gws, aws, n, ht) {
				None => {},
				Some(dt2) => {
					let eval2 = dt2.get_eval();
					if eval2 < 2.0 {
						return Some(dt2);
					} else if eval2 < eval {
						dt = Some(dt2);
						eval = eval2;
					}
				}
			}
		}
	}
	// dont bother checking other words if a 2 was found
	if eval < 2.001 {
		return dt;
	}

	// search top heuristic words and stop
	// on guaranteed next guess (score = 2)
	for gw in top_words(gws, aws, ht, 5) {
		// println!("{}", gw.to_string());
		match solve_given(gw, gws, aws, n, ht) {
			None => {},
			Some(dt2) => {
				let eval2 = dt2.get_eval();
				if eval2 < 2.0001 {
					return Some(dt2);
				} else if eval2 < eval {
					dt = Some(dt2);
					eval = eval2;
				}
			}
		};
	}

	return dt;
}

// best found: salet, 3.425917926565874
fn main() {
	let gws = get_words("data/guess_words").unwrap();
	let aws = get_words("data/answer_words").unwrap();
	let ht = read_heuristic("data/heuristic.csv").unwrap();
	let w = Word::from(&String::from("salet")).unwrap();
	// why is it displaying bad ones???
	// is it bc heuristic table didnt estimate above?
	// for (i,gw) in top_words(&gws, &aws, &ht, 100).iter().enumerate() {
		// println!("{}. {}", i+1, gw.to_string()); 
	// }
	println!("{:?}", solve_given(w, &gws, &aws, 5, &ht));
	// write_data(&"data/data.csv", &pts).unwrap();
}
