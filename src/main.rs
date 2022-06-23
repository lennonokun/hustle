#![allow(dead_code, unused_variables, unused_must_use)]
use std::sync::Mutex;
// use std::error::Error;
use std::time::Instant;
use std::path::Path;
use std::io::{self, Error, ErrorKind};
use main_error::{MainError, MainResult};
use std::env;

mod solve;
use crate::solve::*;
use crate::solve::analysis::*;
mod ds;
use crate::ds::*;
mod game;
use crate::game::Game;

// fix bug where non words are displayed if you guess them on turn 1
fn gen_data(gwb: &WBank, awb: &WBank, hd: &HData, n: i32) {
	let hrm = Mutex::new(HRec::new());
	let gws2 = util::top_words(&gwb, &awb, &hd, n as usize);
	for (i, w) in gws2.iter().enumerate() {
		print!("{}. {}: ", i+1, w.to_string());
		let inst = Instant::now();
		let dt = solve_given(*w, &gwb, &awb, 6, &hd);
		let dur = inst.elapsed().as_millis();
		println!("{}, {:.3}s", dt.unwrap().get_tot(),
						 dur as f64 / 1_000.);
	}
	hrm.into_inner().unwrap()
		.save("data/hdata.csv", "data/hinfs.csv").unwrap();
}

fn solve<P>(s: String, wlen: u8, gwp: P, awp: P, hdp: P)
	-> io::Result<DTree> where P: AsRef<Path>  {
	let gwb = WBank::from(&gwp, wlen)
		.expect("couldn't find guess words!");
	let mut awb = WBank::from(&awp, wlen)
		.expect("couldn't find answer words!");
	let hd = HData::load(hdp)
		.expect("couldn't find heuristic data!");
	let hrm = Mutex::new(HRec::new());

	let mut given = true;
	let mut fbm = FbMap::new();
	let mut w = Word::from_str("aaaaa").unwrap();
	let mut turn = 0;
	for s in s.split(".") {
		if given {
			w = Word::from_str(s).unwrap();
			fbm = util::fb_partition(w, &awb);
			turn += 1;
		} else {
			let fb = Feedback::from_str(s).unwrap();
			awb = fbm.get(&fb).unwrap().clone();
		}
		given = !given;
	}

	// println!("{:?}", awb);
	
	if given {
		solve_state(&gwb, &awb, NGUESSES as i32 - turn, &hd)
			.ok_or(Error::new(ErrorKind::Other, "couldn't make dtree!"))
	} else {
		solve_given(w, &gwb, &awb, NGUESSES as i32 - turn, &hd)
			.ok_or(Error::new(ErrorKind::Other, "couldn't make dtree!"))
	}
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
	let gwp = "data/guess_words";
	let awp = "data/answer_words";
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
	let gwb = WBank::from(&gwp, wlen).expect("couldn't find gwb!");
	let awb = WBank::from(&awp, wlen).expect("couldn't find awb!");
	let hd = HData::load(hdp_in).expect("couldn't find heuristic data!");

	let w1 = Word::from_str("cerne").expect("couldn't make word");
	let w2 = Word::from_str("tweet").expect("couldn't make word");
	let fb = Feedback::from(w1, w2).unwrap();
	println!("{}", fb.to_string());
	match mode.unwrap() {
		"gen" => {
			gen_data(&gwb, &awb, &hd, 100);
		} "play" => {
			let mut game = Game::new();
			game.start();
		} "solve" => {
			// TODO add turns, and count number of words for eval
			let dt = solve(solve_str.unwrap(), wlen, &gwp, &awp, &hdp_in)
				.unwrap();
			dt.pprint(&String::from(""), 0)
		} _ => {}
	}

	Ok(())
}
