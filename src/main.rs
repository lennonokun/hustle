#![allow(unused)]

use std::fs::File;
use std::time::Instant;
use std::sync::Arc;
use std::path::Path;

use hustle::util::*;
use hustle::command::{cli_parse, Commands};
#[cfg(feature = "gen")]
use hustle::analysis::Generator;
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
    },
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
    },
    #[cfg(feature = "gen")]
    Commands::Gen {
      niter,
      out,
      wlen,
      wbp,
      glens,
      alens,
      turns,
      ncacherows,
      ncachecols,
      ntops1,
      ntops2,
      ecut,
    } => {
      let wbank = WBank::load(&wbp, wlen).expect("could not load word bank");
      let cache = Cache::new(ncacherows, ncachecols);

      let defturns = NEXTRA as u32 + wbank.wlen as u32;
      let glens = glens.unwrap_or(Range::new(1, wbank.glen(), true));
      let alens = alens.unwrap_or(Range::new(1, wbank.alen(), true));
      let turns = turns.unwrap_or(Range::new(defturns, defturns, true));
      let mut gen = Generator {
        wbank,
        glens,
        alens,
        turns,
        ncacherows,
        ncachecols,
        ntops1,
        ntops2,
        ecuts: ecut,
        niter,
      };
      gen.run(Path::new(&out));
    },
  }
}
