use std::sync::Mutex;

mod analysis;
use crate::analysis::{HData, HRec};
mod ds;
use crate::ds::*;
mod game;
use crate::game::play;
mod solve;
use crate::solve::*;

// best found: salet, 3.42052836
// out1: salet.BBBYB.drone.BGGBG didnt find prove?
// out2: reast/BYYYY not finding whelk?
fn main() {
	let gws = get_words("data/guess_words").unwrap();
	let aws = get_words("data/answer_words").unwrap();
	let awarr = get_awarr("data/answer_words").unwrap();
	// let hd = HData::load("data/happrox.csv").unwrap();
	// let hr_mut = Mutex::new(HRec::new());
	// let w = Word::from(&String::from("reast")).unwrap();
	play(&gws, &awarr);
	// let dt = solve_given(w, &gws, &aws, 6, &hd, &hr_mut).unwrap();
	// println!("{}", dt.get_eval());

	// dt.pprint(&String::from(""), 1);
	// hr_mut.into_inner().unwrap().save("data/hdata.csv").unwrap();
}
