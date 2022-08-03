#![allow(unused)]

use std::fs::File;
use std::time::Instant;
use std::sync::Arc;
use std::path::Path;

use hustle::util::*;
use hustle::command::{cli_parse, Commands};
#[cfg(feature = "gen")]
use hustle::analysis::{LGen, GGen};
#[cfg(feature = "solve")]
use hustle::solve::{Cache, SData, State, AData};
#[cfg(feature = "play")]
use hustle::game::play;

fn main() {
  let cli = cli_parse();

  match cli.command {
    #[cfg(feature = "play")]
    Commands::Play {} => {
      play();
    }
    #[cfg(feature = "solve")]
    Commands::Solve {
      gamestate,
      alist,
      glist,
      flist,
      dt,
      wbp,
      hdp,
      ldp,
      hard,
      wlen,
      ntops1,
      ntops2,
      turns,
      ecut,
    } => {
      // create state + sdata
      let (gwb, awb) = WBank::from2(wbp, wlen).unwrap();
      let adata = AData::load(&hdp, &ldp).unwrap();
      let cache = Cache::new(64, 16);
      let mut state = State::new2(Arc::new(gwb.data), awb.data, wlen.into(), turns, hard);
      let sd = SData::new(adata, cache, ntops1, ntops2, ecut);

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

      // solve + glist?
      let inst = Instant::now();
      let given = w.is_some();
      let dtree = if !given && glist {
        let ws = state.top_words(&sd);
        let mut scores: Vec<(Word, DTree)> = ws
          .iter()
          .filter_map(|w| Some((*w, state.solve_given(*w, &sd, u32::MAX)?)))
          .collect();
        scores.sort_by_key(|(_w, dt)| dt.get_tot());
        println!("Evaluations:");
        for (i, (w, dt)) in scores.iter().enumerate() {
          println!(
            "{}. {}: {}/{} = {}",
            i + 1,
            w,
            dt.get_tot(),
            dt.get_alen(),
            dt.get_eval(),
          );
        }
        println!();
        Some(scores.remove(0).1)
      } else if !given {
        state.solve(&sd, u32::MAX)
      } else {
        state.solve_given(w.unwrap(), &sd, u32::MAX)
      }
      .expect("couldn't make dtree!");

      if flist {
        println!("Feedbacks:");
        match dtree {
          DTree::Leaf => println!("N/A"),
          DTree::Node {
            tot: _,
            word: _,
            ref fbmap,
          } => {
            for (i, (fb, dt)) in fbmap.iter().enumerate() {
              println!(
                "{}. {}: {}/{} = {}",
                i + 1,
                fb,
                dt.get_tot(),
                dt.get_alen(),
                dt.get_eval(),
              );
            }
          }
        }
        println!();
      }

      // print results
      if let DTree::Node {
        tot,
        word,
        fbmap: _,
      } = dtree
      {
        println!("Solution:");
        println!(
          "{}: {}/{} = {} in {}s",
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
    #[cfg(feature = "gen")]
    Commands::Hgen {
      niter,
      out,
      wlen,
      wbp,
      hdp,
      ldp,
      ntops1,
      ntops2,
      turns,
      ecut,
    } => {
      let (gwb, awb) = WBank::from2(DEFWBP, NLETS as u8).unwrap();
      let adata = AData::load(&hdp, &ldp).unwrap();
      let cache = Cache::new(64, 16);
      let alen_max = awb.len();

      let mut hgen = GGen {
        gwb,
        awb,
        wlen: wlen as u32,
        adata,
        cache,
        alens: Range::new(1, alen_max, true),
        turns: Range::new(6, 6, true),
        ntops1: Range::new(ntops1, ntops1, true),
        ntops2: Range::new(ntops2, ntops2, true),
        ecuts: Range::new(ecut, ecut, true),
        niter,
      };
      hgen.run(Path::new(&out));
    },
    #[cfg(feature = "gen")]
    Commands::Ggen {
      niter,
      out,
      wlen,
      wbp,
      hdp,
      ldp,
      alens,
      ntops1,
      ntops2,
      turns,
      ecut,
    } => {
      let (gwb, awb) = WBank::from2(DEFWBP, NLETS as u8).unwrap();
      let adata = AData::load(&hdp, &ldp).unwrap();
      let cache = Cache::new(64, 16);

      let alens = alens.unwrap_or(Range::new(1, awb.len(), true));
      let mut ggen = GGen {
        gwb,
        awb,
        wlen: wlen as u32,
        adata,
        cache,
        alens,
        turns,
        ntops1,
        ntops2,
        ecuts: ecut,
        niter,
      };
      ggen.run(Path::new(&out));
    },
    #[cfg(feature = "gen")]
    Commands::Lgen {
      niter,
      step,
      out,
      wlen,
      wbp,
      hdp,
      ldp,
      alens,
      ntops1,
      ntops2,
      turns,
      ecut,
    } => {
      let (gwb, awb) = WBank::from2(DEFWBP, NLETS as u8).unwrap();
      let adata = AData::load(&hdp, &ldp).unwrap();
      let cache = Cache::new(64, 16);

      let alens = alens.unwrap_or(Range::new(1, awb.len(), true));
      let mut lgen = LGen {
        niter,
        step,
        gwb,
        awb,
        wlen: wlen as u32,
        adata,
        cache,
        alens,
        turns,
        ntops1,
        ntops2,
        ecut,
      };
      lgen.run(Path::new(&out));
    }
  }
}
