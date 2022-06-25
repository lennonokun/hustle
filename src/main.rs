#![allow(unused, unused_variables, unused_must_use)]
use std::sync::Mutex;
use std::time::Instant;
use std::path::Path;
use std::fs::File;
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

fn gen_data<P>(gwb: &WBank, awb: &WBank, hd: HData,
							 hdop: P, cfg: Config, n: u32)
where P: AsRef<Path> {
	let gws2 = util::top_words(&gwb, &awb, &hd, n as usize);
	for (i, w) in gws2.iter().enumerate() {
		print!("{}. {}: ", i+1, w.to_string());
		let inst = Instant::now();
		let dt = solve_given(*w, &gwb, &awb, 6, &hd, cfg);
		let dur = inst.elapsed().as_millis();
		println!("{}, {:.3}s", dt.unwrap().get_tot(),
						 dur as f64 / 1_000.);
	}

	let mut hrm = hd.hrm.into_inner().unwrap();
	hrm.process(hdop).unwrap();
}

fn solve<P>(s: String, wlen: u8, gwb: &WBank, awb: &WBank,
						hd: &HData, dtp: Option<String>, cfg: Config)
						-> io::Result<()>
where P: AsRef<Path> {
	let hrm = Mutex::new(HRec::new());

	let mut awb2 = awb.clone();
	let mut given = true;
	let mut fbm = FbMap::new();
	let mut w = Word::from_str("aaaaa").unwrap();
	let mut turn = 0u32;
	for s in s.split(".") {
		if given {
			w = Word::from_str(s).unwrap();
			fbm = util::fb_partition(w, &awb2);
			turn += 1;
		} else {
			let fb = Feedback::from_str(s).unwrap();
			awb2 = fbm.get(&fb).unwrap().clone();
		}
		given = !given;
	}

	let dt = if given {
		solve_state(&gwb, &awb2, NGUESSES as u32 - turn, &hd, cfg)
	} else {
		solve_given(w, &gwb, &awb2, NGUESSES as u32 - turn, &hd, cfg)
	}.expect("couldn't make dtree!");

	if let DTree::Node{tot, word, ref fbmap} = dt {
		println!("found {}: {}/{} = {:.6}",
							word.to_string(), tot, awb2.data.len(),
							tot as f64 / awb2.data.len() as f64);
		if let Some(dtp) = dtp {
			let mut f = File::create(dtp)?;
			dt.pprint(&mut f, &"".into(), turn);
		}
	}

	Ok(())
}

// ./hustle
// (play)|(solve <str> <)|(gen <n> <hdp-out>)
// [--(dt|wbp|hdp) <PATH>]*
// [--wlen <WLEN>]
fn main() -> MainResult {
	let mut wlen = 5;
	let mut wbp = String::from("/usr/share/hustle/bank1.csv");
	let mut hdp_in = String::from("/usr/share/hustle/happrox.csv");
	let mut hdp_out = None::<String>;
	let mut mode = None::<&str>;
	let mut dtree_out = None::<String>;
	let mut solve_str = None::<String>;
	let mut gen_num = None::<u32>;
	let mut cfg = Config {ntops: 10, endgcutoff: 15};
	let mut args = env::args().skip(1);

	// parse required arguments
	let first = args.next().expect("Expected an argument!");
	match first.as_str() {
		"play" => {
			mode = Some("play");
		} "solve" => {
			mode = Some("solve");
			// should no argument just mean solve root?
			solve_str = Some(args.next().expect(
				"'solve' requires a secondary argument"));
		} "gen" => {
			mode = Some("gen");
			gen_num = Some(args.next()
										 .expect("'gen' requires a secondary argument")
										 .parse().expect("could not parse gen_num"));
			hdp_out = Some(args.next()
										 .expect("'gen' requires a tertiary argument"));
		} s => {
			return Err(MainError::from(
				Error::new(ErrorKind::Other,
									format!("Invalid argument '{}' found", s))));
		}
	}

	// parse optional arguments
	while let Some(s) = args.next() {
		match s.as_str() {
			"--wlen" => {
				wlen = args.next()
					.expect("'--wlen' requires a secondary argument")
					.parse().expect("could not parse wlen");
			} "--dt" => {
				dtree_out = Some(args.next().expect(
					"'--dt' requires a secondary argument"));
			} "--wbp" => {
				wbp = args.next()
					.expect("'--wbp' requires a secondary argument");
			} "--hdp-in" => {
				hdp_in = args.next()
					.expect("'--hdp-in' requires a secondary argument");
			} "--ntops" => {
				cfg.ntops = args.next()
					.expect("'--ntops' requires a secondary argument")
					.parse().expect("could not parse ntops");
			} "--cutoff" => {
				cfg.endgcutoff = args.next().expect(
					"'--cutoff' requires a secondary argument")
					.parse().expect("could not parse cutoff");
			} s => {
			return Err(MainError::from(
				Error::new(ErrorKind::Other,
									 format!("Invalid argument '{}' found", s))));
			}
		}
	}

	let (gwb, awb) = WBank::from2(wbp, wlen)
		.expect("couldn't load word bank");
	let hd = HData::load(hdp_in)
		.expect("couldn't load heuristic data!");

	match mode.unwrap() {
		"gen" => {
			gen_data(&gwb, &awb, hd, hdp_out.unwrap(),
							 cfg, gen_num.unwrap());
		} "play" => {
			let mut game = Game::new();
			game.start();
		} "solve" => {
			solve::<String>(solve_str.unwrap(), wlen, &gwb, &awb,
						&hd, dtree_out, cfg).unwrap();
		} _ => {}
	}

	Ok(())
}
