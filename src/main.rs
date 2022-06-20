#![allow(dead_code, unused_variables, unused_must_use)]
use std::sync::Mutex;
// use std::error::Error;
use std::time::Instant;
use std::path::Path;
use std::io::{self, Error, ErrorKind};
use main_error::{MainError, MainResult};
use std::env;

mod analysis;
use crate::analysis::{HData, HRec};
mod ds;
use crate::ds::*;
mod game;
use crate::game::Game;
mod solve;
use crate::solve::*;

// best found: salet, 3.42052836
// out1: salet.BBBYB.drone.BGGBG didnt find prove?
// out2: reast/BYYYY not finding whelk?
// fix bug where non words are displayed if you guess them on turn 1
fn gen_data(gwb: &WBank, awb: &WBank, hd: &HData, n: i32) {
	let hrm = Mutex::new(HRec::new());
	let gws2 = top_words(&gwb, &awb, &hd, n as usize);
	for (i, w) in gws2.iter().enumerate() {
		print!("{}. {}: ", i+1, w.to_string());
		let inst = Instant::now();
		let dt = solve_given(*w, &gwb, &awb, 6, &hd, &hrm);
		let dur = inst.elapsed().as_millis();
		println!("{}, {:.3}s", dt.unwrap().get_eval(),
						 dur as f64 / 1_000.);
	}
	hrm.into_inner().unwrap()
		.save("data/hdata.csv", "data/hinfs.csv").unwrap();
}

fn solve_word<P>(s: String, wlen: u8, gwp: P, awp: P, hdp: P)
	-> io::Result<DTree> where P: AsRef<Path>  {
	let w = Word::from(s)
		.expect("couldn't make word");
	let gwb = WBank::from(&gwp, wlen)
		.expect("couldn't find guess words!");
	let awb = WBank::from(&awp, wlen)
		.expect("couldn't find answer words!");
	let hd = HData::load(hdp)
		.expect("couldn't find heuristic data!");
	let hrm = Mutex::new(HRec::new());
	solve_given(w, &gwb, &awb, NGUESSES as i32, &hd, &hrm)
		.ok_or(Error::new(ErrorKind::Other, "couldn't make dtree!"))
}

// ./wordlers
// (gen_data)|(play <n>)|(solve <str>)
// [--(gwp|awp|hdp_in|hdp_out1|hdp_out2) <PATH>]*
fn main() -> MainResult {
	// let dt = solve_word(String::from("SALET"),
										 // "data/guess_words",
										 // "data/answer_words",
										 // "data/happrox.csv");
	// println!("{}", dt.unwrap().get_eval());

	// let mut stdin = io::stdin().lock();
	// let mut stdout = io::stdout().lock();
	// let mut stderr = io::stderr().lock();
	let gwp = "data/guess_words2";
	let awp = "data/answer_words2";
	let hdp_in = "data/happrox.csv";
	let hdp_out1 = "data/hdata.csv";
	let hdp_out2 = "data/hinfs.csv";
	let mut args = env::args().skip(1);
	let mut mode = None::<&str>;
	let mut play_n = None::<u16>;
	let mut solve_str = None::<String>;

	// let mut first = true;
	let first = args.next().expect("Expected an argument!");
	match first.as_str() {
		"gen" => {
			mode = Some("gen");
		} "play" => {
			mode = Some("play");
		} "solve" => {
			mode = Some("solve");
			// should no argument just mean solve root?
			solve_str = Some(args.next()
											 .expect("'solve' requires a secondary argument"));
		} s => {
			return Err(MainError::from(
				Error::new(ErrorKind::Other,
									format!("Invalid argument '{}' found", s))));
		}
	}
	
	if let Some(s) = args.next() {
		return Err(MainError::from(
			Error::new(ErrorKind::Other,
								 format!("Extraneous argument '{}' found", s))));
	}

	let wlen = 5; // FOR NOW
	let w = Word::from_str("SALET").expect("couldn't make word");
	let gwb = WBank::from(&gwp, wlen).expect("couldn't find gwb!");
	let awb = WBank::from(&awp, wlen).expect("couldn't find awb!");
	let hd = HData::load(hdp_in).expect("couldn't find heuristic data!");
	let hrm = Mutex::new(HRec::new());

	match mode.unwrap() {
		"gen" => {
			gen_data(&gwb, &awb, &hd, 100);
		} "play" => {
			let mut game = Game::new(gwp, awp);
			game.start();
		} "solve" => {
			let dt = solve_given(w, &gwb, &awb, NGUESSES as i32, &hd, &hrm);
			dt.unwrap().pprint(&String::from(""), 0)
		} _ => {}
	}

	Ok(())

	// dt.unwrap().pprint(&String::from(""), 1);
					
	// gen_data(&gws, &aws, &hd, 300);
	// let mut game = Game::new(&gws, &awarr);
	// game.start(32);
	// let dt = solve_given(w, &gws, &aws, 6, &hd, &hr_mut).unwrap();
	// println!("{}", dt.get_eval());

}
