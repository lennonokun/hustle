#![allow(unused, unused_variables, unused_must_use)]
use std::sync::Mutex;
use std::time::Instant;
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{self, Error, ErrorKind};
use clap::{Parser, Args, Subcommand, ValueEnum};
use rand::Rng;
use std::io::Write;
use std::env;

mod solve;
use crate::solve::*;
use crate::solve::analysis::*;
mod ds;
use crate::ds::*;
mod game;
use crate::game::Game;

const DEFWBP: &str = "/usr/share/hustle/bank1.csv";
const DEFHDP: &str = "/usr/share/hustle/happrox.csv";

fn gen_data<P>(gwb: &WBank, awb: &WBank, hd: &HData,
							 hdop: P, cfg: Config, niter: u32)
where P: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr> {
	let mut rng = rand::thread_rng();

	// open and write header if new
	let b = Path::new(&hdop).exists();
	let mut out;
	if b {
		out = OpenOptions::new()
			.write(true).append(true)
			.open(hdop).unwrap();
	} else {
		out = File::create(hdop).unwrap();
		writeln!(&mut out, "n,h");
	}

	for i in 0..niter {
		let nsample = rng.gen_range(1..=NWORDS);
		let awb2 = WBank{
			data: awb.pick(&mut rng, nsample as usize),
			wlen: 5
		};
		let dt = solve_state(gwb, &awb2, 6, hd, cfg, u32::MAX);
		if let Some(dt) = dt {
			println!("{}. {}/{}", i+1, dt.get_tot(), nsample);
			writeln!(&mut out, "{},{}", nsample, dt.get_tot());
		}
	}

	// hd.hrm.lock().unwrap().save(hdop);
	// hrm.process(hdop).unwrap();
}

fn solve<P>(s: String, wlen: u8, gwb: &WBank, awb: &WBank,
						hd: &HData, dtp: Option<&P>, list: bool, cfg: Config)
-> io::Result<()> where P: AsRef<Path> {
	let mut awb2 = awb.clone();
	let mut given = true;
	let mut fbm = FbMap::new();
	let mut w = Word::from_str("aaaaa").unwrap();
	let mut turn = 0u32;
	for s in s.split('.') {
		if given {
			w = Word::from_str(s).unwrap();
			fbm = util::fb_partition(w, &awb2);
			turn += 1;
		} else {
			let fb = Feedback::from_str(s).unwrap();
			awb2 = fbm.get(&fb).unwrap_or(
				&WBank{data: Vec::new(), wlen}
			).clone();
		}
		given = !given;
	}

	let dt = if given && list {
		let mut out_dt = None;
		let mut out_tot = u32::MAX;
		let mut scores = Vec::new();
		for w in util::top_words(gwb, &awb2, hd, cfg.ntops as usize) {
			let dt = solve_given(w, gwb, &awb2, NGUESSES as u32 - turn - 1,
													 hd, cfg, u32::MAX);
			if let Some(DTree::Node{tot, word, ref fbmap}) = dt {
				scores.push((word, tot));
				if tot < out_tot {
					out_tot = tot;
					out_dt = dt;
				}
			}
		}

		println!("Listing:");
		scores.sort_by_key(|(w,tot)| *tot);
		for (i,(w,tot)) in scores.iter().enumerate() {
			println!("{}. {}: {}/{} = {:.3}", i+1,
								w.to_string(), tot, awb2.data.len(),
								*tot as f64 / awb2.data.len() as f64);
		}
		println!();
		out_dt
	} else if given {
		solve_state(gwb, &awb2, NGUESSES as u32 - turn,
								hd, cfg, u32::MAX)
	} else {
		solve_given(w, gwb, &awb2, NGUESSES as u32 - turn,
								hd, cfg, u32::MAX)
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
#[clap(version, about)]
struct Cli {
	#[clap(subcommand)]
	command: Commands,
	/// word length
	#[clap(long, default_value_t=5)]
	wlen: u8,
	/// word bank path
	#[clap(long, default_value_t=String::from(DEFWBP))]
	wbp: String,
	/// heuristic data path
	#[clap(long, default_value_t=String::from(DEFHDP))]
	hdp: String,
}

#[derive(Subcommand)]
enum Commands {
	/// Play hustle
	Play,
	/// Solve given game state
	Solve {
		/// the game state to solve from
		#[clap(value_parser)]
		state: String,
		/// list top word evaluations
		#[clap(long)]
		list: bool,
		/// output decision tree to file
		#[clap(long)]
		dt: Option<String>,
		/// the number of top words to check at each state
		#[clap(long, default_value_t=10)]
		ntops: u32,
		/// the maximum number of answer words left for an "endgame"
		#[clap(long, default_value_t=15)]
		cutoff: u32,
	},
	/// Generate heuristic data
	Gen {
		#[clap(value_parser)]
		/// the number of word banks to try
		niter: u32,
		#[clap(value_parser)]
		/// the heuristic data output file
		#[clap(value_parser)]
		hdp_out: String,
		/// the number of top words to check at each state
		#[clap(long, default_value_t=2)]
		ntops: u32,
		/// the maximum number of answer words left for an "endgame"
		#[clap(long, default_value_t=15)]
		cutoff: u32,
	}
}

fn main() {
	let cli = Cli::parse();

	let (gwb, awb) = WBank::from2(cli.wbp, cli.wlen).unwrap();
	let mut hd = HData::load(cli.hdp).unwrap();

	match &cli.command {
		Commands::Play {} => {
			Game::new().start();
		} Commands::Solve {state, list, dt, ntops, cutoff} => {
			let cfg = Config {ntops: *ntops, endgcutoff: *cutoff};
			solve::<String>(state.to_string(), cli.wlen, &gwb, &awb,
											&hd, dt.as_ref(), *list, cfg).unwrap();
		} Commands::Gen {niter, hdp_out, ntops, cutoff} => {
			let cfg = Config {ntops: *ntops, endgcutoff: *cutoff};
			gen_data(&gwb, &awb, &hd, hdp_out, cfg, *niter);
		}
	}
}
