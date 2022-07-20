#![allow(unused)]

extern crate lazy_static;

use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use rand::Rng;
use rand::distributions::Uniform;
use regex::Regex;

use std::str::FromStr;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::time::Instant;
use std::ops::{Range, Add};

mod solve;
use crate::solve::{Cache, SData, HData, State, DataGenerator};
use crate::solve::gen::parse_uniform;
mod ds;
use crate::ds::*;
mod game;
use crate::game::play;

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
    elist: bool,
    /// list potential answers
    #[clap(long)]
    alist: bool,
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
  Gen {
    /// the type of data to generate
    #[clap(value_parser)]
    mode: String,
    /// the number of data points to generate
    #[clap(value_parser)]
    niter: usize,
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
    /// the range of answer lengths to try
    #[clap(long)]
    alens: Option<String>,
    /// the range of turns to try
    #[clap(long)]
    turns: Option<String>,
    /// the range of ntops to try
    #[clap(long)]
    ntops: Option<String>,
    /// the range of ecuts to try
    #[clap(long)]
    ecuts: Option<String>,
    /// the range of ccuts to try
    #[clap(long)]
    ccuts: Option<String>,
  },
}

fn flatten_opt<T>(x: Option<Option<T>>) -> Option<T> {
  match x {
    Some(x) => x,
    None => None
  }
}

fn main() {
  let cli = Cli::parse();

  match cli.command {
    Commands::Play {} => {
      play();
    }
    Commands::Solve {
      gamestate,
      elist,
      alist,
      dt,
      wbp,
      hdp,
      hard,
      wlen,
      ntops,
      ecutoff,
      ccutoff,
    } => {
      // create state + sdata
      let (gwb, awb) = WBank::from2(wbp, wlen).unwrap();
      let hd = HData::load(&hdp).unwrap();
      let cache = Cache::new(16, 4);
      let mut state = State::new(gwb.data, awb.data, wlen.into(), hard);
      let mut sd = SData::new(hd, cache, ntops, ecutoff, ccutoff);

      // parse gamestate
      let mut w: Option<Word> = None;
      let mut turn = 0u32;
      let mut it = gamestate.split('.');
      while let Some(s_a) = it.next() {
        if s_a.is_empty() {
          break;
        }
        turn += 1;
        if let Some(s_b) = it.next() {
          let gw = Word::from_str(s_a).unwrap();
          let fb = Feedback::from_str(s_b).unwrap();
          state = state.fb_follow(gw, fb);
        } else {
          w = Some(Word::from_str(s_a).unwrap());
        }
      }

      // list answers
      if alist {
        println!("Potential Answers:");
        for (i, aw) in state.aws.iter().enumerate() {
          println!("{}. {}", i + 1, aw);
        }
        println!();
      }

      // solve + elist?
      let inst = Instant::now();
      let given = w.is_some();
      let dtree = if !given && elist {
        let ws = state.top_words(&sd);
        let mut scores: Vec<(Word, DTree)> = ws
          .iter()
          .filter_map(|w| Some((*w, state.solve_given(*w, &mut sd, u32::MAX)?)))
          .collect();
        scores.sort_by_key(|(_w, dt)| dt.get_tot());
        println!("Evaluations:");
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
        println!();
        Some(scores.remove(0).1)
      } else if !given {
        state.solve(&mut sd, u32::MAX)
      } else {
        state.solve_given(w.unwrap(), &mut sd, u32::MAX)
      }
      .expect("couldn't make dtree!");

      // print results
      if let DTree::Node {
        tot,
        word,
        fbmap: _,
      } = dtree
      {
        println!("Solution:");
        println!(
          "{}: {}/{} = {:.3} in {:.3}s",
          word.to_string(),
          tot,
          state.aws.len(),
          tot as f64 / state.aws.len() as f64,
          inst.elapsed().as_millis() as f64 / 1000.
        );
        // output dtree
        if let Some(dt) = dt {
          let mut f = File::create(dt).unwrap();
          dtree.pprint(&mut f, &"".into(), turn);
        }
      }
    }
    Commands::Gen {
      mode,
      niter,
      out,
      wlen,
      wbp,
      hdp,
      alens,
      turns,
      ntops,
      ecuts,
      ccuts,
    } => {
      // parse ranges
      let alens = flatten_opt(alens.as_ref().map(parse_uniform::<usize>))
        .unwrap_or(Uniform::new_inclusive(1,NWORDS));
      let turns = flatten_opt(turns.as_ref().map(parse_uniform::<u32>))
        .unwrap_or(Uniform::new_inclusive(1,6));
      let ntops = flatten_opt(ntops.as_ref().map(parse_uniform::<u32>))
        .unwrap_or(Uniform::new_inclusive(1,10));
      let ecuts = flatten_opt(ecuts.as_ref().map(parse_uniform::<u32>))
        .unwrap_or(Uniform::new_inclusive(15,15));
      let ccuts = flatten_opt(ccuts.as_ref().map(parse_uniform::<u32>))
        .unwrap_or(Uniform::new_inclusive(30,30));

      // create + run gen
      let (gwb, awb) = WBank::from2(wbp, wlen).unwrap();
      let hd = HData::load(&hdp).unwrap();
      let cache = Cache::new(16, 4);
      let mut gen = DataGenerator {
        gwb,
        awb,
        wlen,
        hd,
        cache,
        alens,
        turns,
        ntops,
        ecuts,
        ccuts,
        niter
      };
      gen.run(Path::new(&out));
    }
  }
}
