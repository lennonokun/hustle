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
use hustle::solve::{Cache, SolveCommand};
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
      ncacherows,
      ncachecols,
      ntops1,
      ntops2,
      ecut,
    } => {
      let wbank = WBank::load(&wbp, wlen).unwrap();
      let glen = wbank.glen();
      let alen = wbank.alen();

      let mut hgen = GGen {
        wbank,
        glens: Range::new(glen, glen, true),
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
      glens,
      alens,
      ncacherows,
      ncachecols,
      ntops1,
      ntops2,
      ecut,
    } => {
      let wbank = WBank::load(&wbp, wlen).expect("could not load word bank");
      let cache = Cache::new(ncacherows, ncachecols);

      let glens = glens.unwrap_or(Range::new(1, wbank.glen(), true));
      let alens = alens.unwrap_or(Range::new(1, wbank.alen(), true));
      let mut ggen = GGen {
        wbank,
        glens,
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
      alens,
      ncacherows,
      ncachecols,
      ntops1,
      ntops2,
      ecut,
    } => {
      let wbank = WBank::load(&wbp, wlen).unwrap();

      let alens = alens.unwrap_or(Range::new(1, wbank.alen(), true));
      let mut lgen = LGen {
        niter,
        step,
        wbank,
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
