#![allow(unused)]

use std::fs::File;
use std::time::Instant;
use std::sync::Arc;
use std::path::Path;
use std::io::stdout;

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
      wbank,
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
        wbank,
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
    }, Commands::Diff {
      dtree1,
      dtree2,
    } => {
      let dt1 = DTree::load(&dtree1)
        .expect(&format!("could not load dtree at '{dtree1}'"));
      let dt2 = DTree::load(&dtree2)
        .expect(&format!("could not load dtree at '{dtree2}'"));
      let mut stdout = stdout().lock();
      DTree::print_diff(&mut stdout, &dt1, &dt2)
        .expect("incompatible decision trees");
    },
    #[cfg(feature = "gen")]
    Commands::Gen {
      niter,
      out,
      wlen,
      wbank,
      glens,
      alens,
      turns,
      ncacherows,
      ncachecols,
      ntops1,
      ntops2,
      ecut,
    } => {
      let wbank = WBank::load(&wbank, wlen).expect("could not load word bank");
      let cache = Cache::new(ncacherows, ncachecols);

      let glens = glens.unwrap_or(Range::new(1, wbank.glen(), true));
      let alens = alens.unwrap_or(Range::new(1, wbank.alen(), true));
      let turns = turns.unwrap_or(Range::new(6, 6, true));
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
