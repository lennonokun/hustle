#![allow(unused, unused_variables, unused_must_use)]
use std::sync::Mutex;
use std::time::Instant;
use std::path::Path;
use std::fs::File;
use std::io::{self, Error, ErrorKind};
use clap::{Parser, Args, Subcommand, ValueEnum};
use std::env;

mod solve;
use crate::solve::*;
use crate::solve::analysis::*;
mod ds;
use crate::ds::*;
mod game;
use crate::game::Game;

const DEFWBP: &'static str = "/usr/share/hustle/bank1.csv";
const DEFHDP: &'static str = "/usr/share/hustle/happrox.csv";

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
						hd: &HData, dtp: Option<&P>, list: bool, cfg: Config)
-> io::Result<()> where P: AsRef<Path> {
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

	let dt = if given && list {
		let mut out_dt = None;
		let mut out_tot = u32::MAX;
		println!("Listing:");
		for w in util::top_words(gwb, &awb2, hd, cfg.ntops as usize) {
			let dt = solve_given(w, &gwb, &awb2, NGUESSES as u32 - turn - 1,
													 &hd, cfg);
			if let Some(DTree::Node{tot, word, ref fbmap}) = dt {
				println!("{}: {}/{} = {:.3}",
								 word.to_string(), tot, awb2.data.len(),
								 tot as f64 / awb2.data.len() as f64);
				if tot < out_tot {
					out_tot = tot;
					out_dt = dt;
				}
			}
		}
		println!();
		out_dt
	} else if given {
		solve_state(&gwb, &awb2, NGUESSES as u32 - turn, &hd, cfg)
	} else {
		solve_given(w, &gwb, &awb2, NGUESSES as u32 - turn, &hd, cfg)
	}.expect("couldn't make dtree!");

	if let DTree::Node{tot, word, ref fbmap} = dt {
		println!("found {}: {}/{} = {:.3}",
							word.to_string(), tot, awb2.data.len(),
							tot as f64 / awb2.data.len() as f64);
		if let Some(dtp) = dtp {
			let mut f = File::create(dtp)?;
			dt.pprint(&mut f, &"".into(), turn);
		}
	}

	Ok(())
}

#[derive(Parser)]
#[clap(author, version, about, long_about=None)]
struct Cli {
	#[clap(subcommand)]
	command: Commands,
	#[clap(long, default_value_t=5)]
	wlen: u8,
	#[clap(long, default_value_t=String::from(DEFWBP))]
	wbp: String,
	#[clap(long, default_value_t=String::from(DEFHDP))]
	hdp: String,
	#[clap(long, default_value_t=10)]
	ntops: u32,
	#[clap(long, default_value_t=15)]
	cutoff: u32,
}

#[derive(Subcommand)]
enum Commands {
	Play {
	}, Solve {
		#[clap(value_parser)]
		state: String,
		#[clap(long)]
		list: bool,
		#[clap(long)]
		dt: Option<String>,
	}, Gen {
		#[clap(value_parser)]
		niter: u32,
		#[clap(value_parser)]
		hdp_out: String,
	}
}

fn main() {
	let cli = Cli::parse();

	let (gwb, awb) = WBank::from2(cli.wbp, cli.wlen).unwrap();
	let hd = HData::load(cli.hdp).unwrap();
	let cfg = Config {ntops: cli.ntops, endgcutoff: cli.cutoff};

	match &cli.command {
		Commands::Play {} => {
			Game::new().start();
		} Commands::Solve {state, list, dt} => {
			solve::<String>(state.to_string(), cli.wlen, &gwb, &awb,
											&hd, dt.as_ref(), *list, cfg).unwrap();
		} Commands::Gen {niter, hdp_out} => {
			gen_data(&gwb, &awb, hd, hdp_out, cfg, *niter);
		}
	}
}
