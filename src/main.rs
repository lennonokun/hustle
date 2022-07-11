#![allow(unused, unused_variables, unused_must_use)]
#[macro_use]
use clap::{Args, Parser, Subcommand, ValueEnum, clap_app};
use rand::Rng;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::{self, Error, ErrorKind};
use std::path::Path;
use std::time::Instant;
use rand::prelude::*;
use rayon::prelude::*;
use std::sync::Mutex;

mod solve;
use crate::solve::{State, Config, HData, Cache};
mod ds;
use crate::ds::*;
mod game;
use crate::game::game;

const DEFWBP: &str = "/usr/share/hustle/bank1.csv";
const DEFHDP: &str = "/usr/share/hustle/happrox.csv";

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// play hustle
  Play,
  /// solve game state
  Solve {
    /// the game state to solve from
    #[clap(value_parser, default_value = "")]
    gamestate: String,
    /// list top word evaluations
    #[clap(long)]
    list: bool,
    /// output decision tree to file
    #[clap(long)]
    dt: Option<String>,
    /// word length
    #[clap(long, default_value_t = 5)]
    wlen: u8,
    /// word bank path
    #[clap(long, default_value_t=String::from(DEFWBP))]
    wbp: String,
    /// heuristic data path
    #[clap(long, default_value_t=String::from(DEFHDP))]
    hdp: String,
    /// play in hard mode
    #[clap(long)]
    hard: bool,
    /// the number of top words to check at each state
    #[clap(long, default_value_t = 10)]
    ntops: u32,
    /// the maximum number of answer words left for an "endgame"
    #[clap(long, default_value_t = 15)]
    ecutoff: u32,
    /// the minimum number of answers word left to cache
    #[clap(long, default_value_t = 30)]
    ccutoff: u32,
  },
  /// generate heuristic data
  Hgen {
    /// the number of word banks to try
    #[clap(value_parser)]
    niter: u32,
    /// the file to output data to
    #[clap(value_parser)]
    out: String,
    /// word length
    #[clap(long, default_value_t = 5)]
    wlen: u8,
    /// word bank path
    #[clap(long, default_value_t=String::from(DEFWBP))]
    wbp: String,
    /// heuristic data path
    #[clap(long, default_value_t=String::from(DEFHDP))]
    hdp: String,
  },
  /// generate general analysis data
  Agen {
    /// the number of word banks to try
    #[clap(value_parser)]
    niter: u32,
    /// the file to output data to
    #[clap(value_parser)]
    out: String,
    /// word length
    #[clap(long, default_value_t = 5)]
    wlen: u8,
    /// word bank path
    #[clap(long, default_value_t=String::from(DEFWBP))]
    wbp: String,
    /// heuristic data path
    #[clap(long, default_value_t=String::from(DEFHDP))]
    hdp: String,
  }
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Commands::Play {} => {
      game();
    }
    Commands::Solve {
      gamestate,
      list,
      dt,
      wbp,
      hdp,
      hard,
      wlen,
      ntops,
      ecutoff,
      ccutoff,
    } => {
      let (gwb, awb) = WBank::from2(wbp, wlen).unwrap();
      let hd = HData::load(&hdp).unwrap();
      let cache = Cache::new(16, 4);
      let mut state = State::new(gwb.data, awb.data, wlen.into(), hard);
      let mut cfg = Config::new(hd, cache, ntops, ecutoff, ccutoff);

      // parse gamestate
      let mut w: Option<Word> = None;
      let mut turn = 0u32;
      let mut it = gamestate.split('.');
      while let Some(s_a) = it.next() {
        if s_a.is_empty() {break}
        turn += 1;
        if let Some(s_b) = it.next() {
          let gw = Word::from_str(s_a).unwrap();
          let fb = Feedback::from_str(s_b).unwrap();
          state = state.fb_follow(gw, fb);
        } else {
          w = Some(Word::from_str(s_a).unwrap());
        }
      }

      // solve + list?
      let given = w.is_some();
      let dtree = if !given && list {
        let ws = state.top_words(&cfg);
        let mut scores: Vec<(Word, DTree)> = ws
          .iter()
          .filter_map(|w| Some((*w, state.solve_given(*w, &mut cfg, u32::MAX)?)))
          .collect();
        scores.sort_by_key(|(w, dt)| dt.get_tot());
        println!("Listing:");
        for (i, (w, dt)) in scores.iter().enumerate() {
          println!(
            "{}. {}: {}/{} = {:.3}",
            i + 1,
            w.to_string(),
            dt.get_tot(),
            state.aws.len(),
            dt.get_tot() as f64 / state.aws.len() as f64
          );
        }
        scores.pop().map(|(w, dt)| dt)
      } else if !given {
        state.solve(&mut cfg, u32::MAX)
      } else {
        state.solve_given(w.unwrap(), &mut cfg, u32::MAX)
      }
      .expect("couldn't make dtree!");

      // print results
      if let DTree::Node {
        tot,
        word,
        ref fbmap,
      } = dtree
      {
        println!(
          "found {}: {}/{} = {:.3}",
          word.to_string(),
          tot,
          state.aws.len(),
          tot as f64 / state.aws.len() as f64
        );
        // output dtree
        if let Some(dt) = dt {
          let mut f = File::create(dt).unwrap();
          dtree.pprint(&mut f, &"".into(), turn);
        }
      }
    }
    Commands::Hgen {niter, out, wlen, wbp, hdp} => {
      // get wbanks + config
      let (gwb, awb) = WBank::from2(DEFWBP, NLETS as u8).unwrap();
      let hd = HData::load(DEFHDP).unwrap();
      let cache = Cache::new(16, 4);
      let cfg = Config::new(hd, cache, 2, 15, 30);
      
      // open and write if new
      let mut f;
      if Path::new(&out).exists() {
        f = OpenOptions::new()
          .write(true)
          .append(true)
          .open(out)
          .unwrap();
      } else {
        f = File::create(out).unwrap();
        writeln!(&mut f, "alen,tot");
      }

      // solve randomly sized states in parallel
      let f = Mutex::new(f);
      let i = Mutex::new(1);
      (0..niter).into_par_iter().for_each(|_| {
        // pick aws
        let mut rng = rand::thread_rng();
        let alen = rng.gen_range(1..=NWORDS);
        let aws2 = awb.pick(&mut rng, alen as usize);
        
        let s = State::new2(gwb.data.clone(), aws2, awb.wlen.into(), NGUESSES as u32, false);
        let mut cfg = cfg.clone();
        if let Some(dt) = s.solve(&mut cfg, u32::MAX) {
          let mut f = f.lock().unwrap();
          let mut i = i.lock().unwrap();
          println!("{}. alen: {}, tot: {}", i, alen, dt.get_tot());
          writeln!(f, "{},{}", alen, dt.get_tot());
          *i += 1;
        }
      });
    }
    Commands::Agen {niter, out, wlen, wbp, hdp} => {
      // get constants
      let (gwb, awb) = WBank::from2(wbp, NLETS as u8).unwrap();
      let hd = HData::load(&hdp).unwrap();
      let cache = Cache::new(16, 4);

      // open and write header if new
      let mut f;
      if Path::new(&out).exists() {
        f = OpenOptions::new()
          .write(true)
          .append(true)
          .open(out)
          .unwrap();
      } else {
        f = File::create(out).unwrap();
        writeln!(&mut f, "alen,tot,time,turns,mode,ntops,ecut,ccut");
      }

      let f = Mutex::new(f);
      let i = Mutex::new(1);
      (0..niter).into_par_iter().for_each(|_| {
        // pick random features
        let mut rng = rand::thread_rng();
        let alen = rng.gen_range(1..=NWORDS);
        let aws2 = awb.pick(&mut rng, alen as usize);
        let turns = rng.gen_range(1..=6);
        let ntops = rng.gen_range(1..=8);
        let ecut = 15;
        let ccut = 30;
        let hard = false; // FOR NOW ALWAYS EASY BC CACHE DONT CHECK GWS

        // generate state
        let s = State::new2(gwb.data.clone(), aws2, awb.wlen.into(), turns, false);
        let mut cfg = Config::new(hd.clone(), cache.clone(), ntops, ecut, ccut);

        // solve and time
        let instant = Instant::now();
        let dt = s.solve(&mut cfg, u32::MAX);
        let time = instant.elapsed().as_millis();
        // MAYBE MAKE IT u32 max? or nan (but make float)
        let tot = dt.map_or(0, |dt| dt.get_tot());

        let mut i = i.lock().unwrap();
        let mut f = f.lock().unwrap();
        println!("{}. {},{},{},{},{},{},{},{}",
                 *i, alen, tot, time, turns, if hard {"H"} else {"E"},
                 cfg.ntops, cfg.endgcutoff, cfg.cachecutoff);
        writeln!(f, "{},{},{},{},{},{},{},{}",
                 alen, tot, time, turns, if hard {"H"} else {"E"},
                 cfg.ntops, cfg.endgcutoff, cfg.cachecutoff);
        *i += 1;
      });
    }
  }
}
