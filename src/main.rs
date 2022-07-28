#![allow(unused)]

extern crate lazy_static;

use lazy_static::lazy_static;
use rand::Rng;
use rand::distributions::Uniform;
use regex::Regex;
use rayon::prelude::*;

use std::sync::Mutex;
use std::str::FromStr;
use std::fs::{File, OpenOptions};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

mod ds;
use crate::ds::*;
mod command;
use crate::command::{cli_parse, Commands};
mod analysis;
use analysis::{SGen, Range, HData};
mod solve;
use crate::solve::{Cache, SData, State};
mod game;
use crate::game::play;


fn flatten_opt<T>(x: Option<Option<T>>) -> Option<T> {
  match x {
    Some(x) => x,
    None => None
  }
}

fn main() {
  let cli = cli_parse();

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
      turns,
      ecut,
    } => {
      // create state + sdata
      let (gwb, awb) = WBank::from2(wbp, wlen).unwrap();
      let hd = HData::load(&hdp).unwrap();
      let cache = Cache::new(64, 16);
      let mut state = State::new2(gwb.data, awb.data, wlen.into(), turns, hard);
      let mut sd = SData::new(hd, cache, ntops, ecut);

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
    Commands::Hgen {
      niter,
      out,
      wlen,
      wbp,
      hdp,
      ntops,
      turns,
      ecut,
    } => {
      let (gwb, awb) = WBank::from2(DEFWBP, NLETS as u8).unwrap();
      let hd = HData::load(DEFHDP).unwrap();
      let cache = Cache::new(64, 16);

      let mut sgen = SGen {
        gwb,
        awb,
        wlen: wlen as u32,
        hd,
        cache,
        alens: Range::new(1, 2309, true),
        turns: Range::new(6, 6, true),
        ntops: Range::new(turns, turns, true),
        ecuts: Range::new(ecut, ecut, true),
        niter,
      };
      sgen.run(Path::new(&out));
    },
    Commands::Ggen {
      niter,
      out,
      wlen,
      wbp,
      hdp,
      ntops,
      turns,
      ecut,
    } => {
      let (gwb, awb) = WBank::from2(DEFWBP, NLETS as u8).unwrap();
      let hd = HData::load(DEFHDP).unwrap();
      let cache = Cache::new(64, 16);

      let alen_max = awb.len();
      let mut sgen = SGen {
        gwb,
        awb,
        wlen: wlen as u32,
        hd,
        cache,
        alens: Range::new(1, alen_max, true),
        turns,
        ntops,
        ecuts: ecut,
        niter,
      };
      sgen.run(Path::new(&out));
    },
  }
}
