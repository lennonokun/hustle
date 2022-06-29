use crate::ds::*;
use crate::solve::analysis::HData;

// get feedback partitions
pub fn fb_partition(gw: Word, awb: &WBank) -> FbMap<WBank> {
	let mut map = FbMap::new();
	for aw in &awb.data {
		let fb = Feedback::from(gw, *aw).unwrap();
		let wb2 : &mut WBank =
			map.entry(fb).or_insert_with(WBank::new);
		wb2.data.push(*aw);
	};
	map
}

// get feedback partition counts
pub fn fb_counts(gw: Word, awb: &WBank) -> FbMap<u32> {
	let mut map = FbMap::new();
	for aw in &awb.data {
		let fb = Feedback::from(gw, *aw).unwrap();
		*map.entry(fb).or_insert(0) += 1
	};
	map
}

// apply precalculated heuristic to partition sizes (lower is better)
pub fn heuristic(gw: Word, awb: &WBank, hd: &HData) -> f64 {
	let h = fb_counts(gw, awb).iter()
		.map(|(_, n)| hd.get_approx(*n as usize))
		.sum();
	if awb.contains(gw) {h - 1.} else {h}
}

// get top n words based off of heuristic
pub fn top_words(gwb: &WBank, awb: &WBank, hd: &HData, n: usize)
	-> Vec<Word> {
	let mut tups : Vec<(Word, f64)> = gwb.data.iter()
		.map(|gw| (*gw, heuristic(*gw, awb, hd)))
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

pub fn reduce_words(gw: Word, gwb: &WBank) -> WBank {
	let data2 = gwb.data.iter()
		// why double?
		.filter(|gw2| common_letters(gw, **gw2) <= 1)
		.cloned()
		.collect();
	WBank {data:data2, wlen:gwb.wlen}
}
