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
      ncacherows,
      ncachecols,
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
        turns,
        wlen,
        ncacherows,
        ncachecols,
        ntops1,
        ntops2,
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
      ncacherows,
      ncachecols,
      ntops1,
      ntops2,
      ecut,
    } => {
      let wbank = WBank::load(&wbp, wlen).unwrap();
      let adata = AData::load(&hdp, &ldp).unwrap();
      let alen = wbank.alen();

      let mut hgen = GGen {
        wbank,
        adata,
        alens: Range::new(1, alen, true),
        ncacherows,
        ncachecols,
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
      ncacherows,
      ncachecols,
      ntops1,
      ntops2,
      ecut,
    } => {
      let wbank = WBank::load(&wbp, wlen).unwrap();
      let adata = AData::load(&hdp, &ldp).unwrap();
      let cache = Cache::new(ncacherows, ncachecols);

      let alens = alens.unwrap_or(Range::new(1, wbank.alen(), true));
      let mut ggen = GGen {
        wbank,
        adata,
        alens,
        ncacherows,
        ncachecols,
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
      ncacherows,
      ncachecols,
      ntops1,
      ntops2,
      ecut,
    } => {
      let wbank = WBank::load(&wbp, wlen).unwrap();
      let adata = AData::load(&hdp, &ldp).unwrap();

      let alens = alens.unwrap_or(Range::new(1, wbank.alen(), true));
      let mut lgen = LGen {
        niter,
        step,
        wbank,
        adata,
        alens,
        ncacherows,
        ncachecols,
        ntops1,
        ntops2,
        ecut,
      };
      lgen.run(Path::new(&out));
    }
  }
}
