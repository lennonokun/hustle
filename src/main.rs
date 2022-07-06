#![allow(unused, unused_variables, unused_must_use)]
use clap::{Args, Parser, Subcommand, ValueEnum};
use rand::Rng;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::{self, Error, ErrorKind};
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

mod solve;
use crate::solve::analysis::*;
use crate::solve::*;
mod ds;
use crate::ds::*;
mod game;
use crate::game::Game;

const DEFWBP: &str = "/usr/share/hustle/bank1.csv";
const DEFHDP: &str = "/usr/share/hustle/happrox.csv";

fn gen_data<P>(gwb: WBank, awb: WBank, hd: &HData, hdop: P, cfg: Config, niter: u32)
where
  P: AsRef<Path> + std::convert::AsRef<std::ffi::OsStr>, {
  let mut rng = rand::thread_rng();

  // open and write header if new
  let b = Path::new(&hdop).exists();
  let mut out;
  if b {
    out = OpenOptions::new()
      .write(true)
      .append(true)
      .open(hdop)
      .unwrap();
  } else {
    out = File::create(hdop).unwrap();
    writeln!(&mut out, "n,h,m");
  }

  for i in 0..niter {
    let nsample = rng.gen_range(1..=NWORDS);
    let aws2 = awb.pick(&mut rng, nsample as usize);
    let s = State::new(gwb.data.clone(), aws2, awb.wlen.into());
    let dt = s.solve(&cfg, hd, u32::MAX);
    if let Some(dt) = dt {
      let mode = if cfg.hard {"H"} else {"E"};
      println!("{}. {}/{}", i + 1, dt.get_tot(), nsample);
      writeln!(&mut out, "{},{},{}", nsample, dt.get_tot(), mode);
    }
  }
}

fn solve<P>(
  s: String, wlen: u8, gwb: WBank, awb: WBank, hd: &HData, dtp: Option<&P>, list: bool, cfg: Config,
) -> io::Result<()>
where
  P: AsRef<Path>, {
  let mut given = true;
  let mut fbm = FbMap::new();
  let mut state = State::new(gwb.data, awb.data, gwb.wlen.into());
  let mut w = Word::from_str("aaaaa").unwrap();
  let mut turn = 0u32;
  for s in s.split('.') {
    if given {
      w = Word::from_str(s).unwrap();
      fbm = state.fb_partition(&w, &cfg);
      turn += 1;
    } else {
      let fb = Feedback::from_str(s).unwrap();
      state = fbm.remove(&fb).unwrap();
    }
    given = !given;
  }

  let dt = if given && list {
    let ws = state.top_words(&cfg, &hd);
    let mut scores: Vec<(Word, DTree)> = ws
      .iter()
      .filter_map(|w| Some((*w, state.solve_given(*w, &cfg, &hd, u32::MAX)?)))
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
  } else if given {
    state.solve(&cfg, hd, u32::MAX)
  } else {
    state.solve_given(w, &cfg, hd, u32::MAX)
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
    //            fb.to_string(),
    //            word.to_string(),
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
    #[clap(value_parser)]
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
    cutoff: u32,
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
    cutoff: u32,
    /// play in hard mode
    #[clap(long)]
    hard: bool,
  },
}

fn main() {
  let cli = Cli::parse();

  let (gwb, awb) = WBank::from2(cli.wbp, cli.wlen).unwrap();
  let mut hd = HData::load(cli.hdp).unwrap();

  match &cli.command {
    Commands::Play {} => {
      Game::new().start();
    }
    Commands::Solve {
      state,
      list,
      dt,
      ntops,
      cutoff,
      hard,
    } => {
      let cfg = Config {
        ntops: *ntops,
        endgcutoff: *cutoff,
        hard: *hard,
      };
      solve::<String>(
        state.to_string(),
        cli.wlen,
        gwb,
        awb,
        &hd,
        dt.as_ref(),
        *list,
        cfg,
      )
      .unwrap();
    }
    Commands::Gen {
      niter,
      hdp_out,
      ntops,
      cutoff,
      hard,
    } => {
      let cfg = Config {
        ntops: *ntops,
        endgcutoff: *cutoff,
        hard: *hard,
      };
      gen_data(gwb, awb, &hd, hdp_out, cfg, *niter);
    }
  }
}
