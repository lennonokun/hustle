#![allow(dead_code, unused_variables, unused_must_use)]
// use std::sync::Mutex;
// use std::io;

mod analysis;
// use crate::analysis::{HData, HRec};
mod ds;
use crate::ds::*;
mod game;
use crate::game::Game;
mod solve;
// use crate::solve::*;

// TODO:
// fix warnings fixed by allow
// make start + end screen
// allow for deletion of completed fbcols 
// generally refactor

// best found: salet, 3.42052836
// out1: salet.BBBYB.drone.BGGBG didnt find prove?
// out2: reast/BYYYY not finding whelk?
// fix bug where non words are displayed if you guess them on turn 1
fn main() {
	let gws = get_words("data/guess_words").unwrap();
	let aws = get_words("data/answer_words").unwrap();
	let awarr = get_awarr("data/answer_words").unwrap();
	// let hd = HData::load("data/happrox.csv").unwrap();
	// let hr_mut = Mutex::new(HRec::new());
	// let w = Word::from(&String::from("reast")).unwrap();
	let mut game = Game::new(&gws, &awarr);
	game.start(16);
	// let dt = solve_given(w, &gws, &aws, 6, &hd, &hr_mut).unwrap();
	// println!("{}", dt.get_eval());

	// dt.pprint(&String::from(""), 1);
	// hr_mut.into_inner().unwrap().save("data/hdata.csv").unwrap();
}
