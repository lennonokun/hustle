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
use hustle::solve::{Cache, AData, SolveCommand};
#[cfg(feature = "play")]
use hustle::game::play;

fn main() {
  let cli = cli_parse();

  match cli_parse().command {
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
      let scmd = SolveCommand {
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
      };
      scmd.run();
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
