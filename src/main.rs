#![allow(unused, unused_variables, unused_must_use)]
use clap::{Args, Parser, Subcommand, ValueEnum};
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

fn gen_data<P>(wbp: &P, hdop: &P, cfg: Config, niter: u32, hard: bool)
where P: AsRef<Path>+std::convert::AsRef<std::ffi::OsStr> {
  // read wbp + parse mode
  let (gwb, awb) = WBank::from2(wbp, NLETS as u8).unwrap();
  let mode = if hard {"H"} else {"E"};


  // open and write header if new
  let mut out;
  if Path::new(hdop).exists() {
    out = OpenOptions::new()
      .write(true)
      .append(true)
      .open(hdop)
      .unwrap();
  } else {
    out = File::create(hdop).unwrap();
    writeln!(&mut out, "alen,tot,time,mode,ntops,ecut,ccut");
  }

  let out = Mutex::new(out);
  let i = Mutex::new(1);
  (0..niter).into_par_iter().for_each(|_| {
    let mut rng = rand::thread_rng();

    let alen = rng.gen_range(1..=NWORDS);
    let aws2 = awb.pick(&mut rng, alen as usize);
    let mut cfg = cfg.clone();
    let s = State::new(gwb.data.clone(), aws2, awb.wlen.into(), false);

    let instant = Instant::now();
    let dt = s.solve(&mut cfg, u32::MAX);
    let time = instant.elapsed().as_millis();
    let tot = dt.map_or(0, |dt| dt.get_tot());

    let mut i = i.lock().unwrap();
    println!("{}. {}/{}", i, tot, alen);
    writeln!(out.lock().unwrap(), "{},{},{},{},{},{},{}",
             alen, tot, time, mode, cfg.ntops,
             cfg.endgcutoff, cfg.cachecutoff);
    *i += 1;
  });
}

fn solve<P>(
  wbp: &P, s: String, wlen: u8, mut cfg: Config,
  dtp: Option<&P>, hard: bool, list: bool) -> io::Result<()>
where P: AsRef<Path>, {
  let (gwb, awb) = WBank::from2(wbp, wlen).unwrap();
  let mut state = State::new(gwb.data, awb.data, wlen.into(), hard);
  let mut w: Option<Word> = None;
  let mut turn = 0u32;
  let mut it = s.split('.');
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

  let given = w.is_some();
  let dt = if !given && list {
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

  if let DTree::Node {
    tot,
    word,
    ref fbmap,
  } = dt
  {
    println!(
      "found {}: {}/{} = {:.3}",
      word.to_string(),
      tot,
      state.aws.len(),
      tot as f64 / state.aws.len() as f64
    );
    if let Some(dtp) = dtp {
      let mut f = File::create(dtp)?;
      dt.pprint(&mut f, &"".into(), turn);
    }

    //    let mut v: Vec<(&Feedback, &DTree)> = fbmap.iter().collect();
    //    v.sort_by_key(|(fb, dt)| -(dt.get_tot() as i32));
    //    for (i, (fb, dt)) in v.iter().enumerate() {
    //      match dt {
    //        DTree::Leaf => {}
    //        DTree::Node { tot, word, fbmap } => {
    //          println!(
    //            "{}. {}: {}, {}",
    //            i + 1,
    //            fb,
    //            word,
    //            tot
    //          );
    //        }
    //      }
    //    }
  }

  Ok(())
}

#[derive(Parser)]
#[clap(version, about)]
struct Cli {
  #[clap(subcommand)]
  command: Commands,
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

#[derive(Subcommand)]
enum Commands {
  /// Play hustle
  Play,
  /// Solve given game state
  Solve {
    /// the game state to solve from
    #[clap(value_parser, default_value = "")]
    state: String,
    /// list top word evaluations
    #[clap(long)]
    list: bool,
    /// output decision tree to file
    #[clap(long)]
    dt: Option<String>,
    /// the number of top words to check at each state
    #[clap(long, default_value_t = 10)]
    ntops: u32,
    /// the maximum number of answer words left for an "endgame"
    #[clap(long, default_value_t = 15)]
    ecutoff: u32,
    /// the minimum number of answers word left to cache
    #[clap(long, default_value_t = 30)]
    ccutoff: u32,
    /// play in hard mode
    #[clap(long)]
    hard: bool,
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
    #[clap(long, default_value_t = 2)]
    ntops: u32,
    /// the maximum number of answer words left for an "endgame"
    #[clap(long, default_value_t = 15)]
    ecutoff: u32,
    /// the minimum number of answers word left to cache
    #[clap(long, default_value_t = 30)]
    ccutoff: u32,
    /// play in hard mode
    #[clap(long)]
    hard: bool,
  },
}

fn main() {
  let cli = Cli::parse();

  match &cli.command {
    Commands::Play {} => {
      game();
    }
    Commands::Solve {
      state,
      list,
      dt,
      ntops,
      ecutoff,
      ccutoff,
      hard,
    } => {
      let hd = HData::load(&cli.hdp).unwrap();
      let cache = Cache::new(16, 4);
      let cfg = Config::new(hd, cache, *ntops, *ecutoff, *ccutoff);
      solve::<String>(
        &cli.wbp,
        state.to_string(),
        cli.wlen,
        cfg,
        dt.as_ref(),
        *hard,
        *list,
      )
      .unwrap();
    }
    Commands::Gen {
      niter,
      ref hdp_out,
      ntops,
      ecutoff,
      ccutoff,
      hard,
    } => {
      let hd = HData::load(&cli.hdp).unwrap();
      let cache = Cache::new(16, 4);
      let cfg = Config::new(hd, cache, *ntops, *ecutoff, *ccutoff);

      gen_data(&cli.wbp, hdp_out, cfg, *niter, *hard);
    }
  }
}
