use std::time::Instant;
use std::fs::File;
use std::sync::Arc;

use tabled::{Tabled, Table, Style};
use rayon::prelude::*;

use crate::util::*;
use super::{State, SData, Cache};

#[derive(Tabled)]
struct GResult {
  i: usize,
  guess: Word,
  total: u32,
  alen: u32,
  #[tabled(display_with = "format_eval")]
  eval: f64,
}

#[derive(Tabled)]
struct AResult {
  i: usize,
  answer: Word,
}

#[derive(Tabled)]
struct FResult {
  i: usize,
  feedback: Feedback,
  total: u32,
  alen: u32,
  #[tabled(display_with = "format_eval")]
  eval: f64,
}

fn format_eval(eval: &f64) -> String {
  format!("{eval:.4}")
}

pub struct SolveCommand {
  /// the game state to solve from
  pub gamestate: String,
  /// list potential answers
  pub alist: bool,
  /// list potential feedbacks and their evaluations
  pub flist: bool,
  /// list top guess evaluations
  pub glist: bool,
  /// output decision tree to file
  pub dt: Option<String>,
  /// word length
  pub wlen: u8,
  /// word bank path
  pub wbp: String,
  /// heuristic data path
  pub hard: bool,
  /// the number of rows/sets in the cache
  pub ncacherows: usize,
  /// the number of columns in the cache
  pub ncachecols: usize,
  /// the number of top soft heuristic words to try
  pub ntops1: u32,
  /// the number of top hard heuristic words to try
  pub ntops2: u32,
  /// the maximum number of turns to solve in
  pub turns: Option<u32>,
  /// the maximum number of answer words left for an "endgame"
  pub ecut: u32,
}

impl SolveCommand {
  fn load_and_parse(&self) -> (State, SData, Option<Word>, u32) {
    // load data
    let wbank = WBank::load(&self.wbp, self.wlen)
      .expect("could not load word bank!");
    let cache = Cache::new(self.ncacherows, self.ncachecols);
    let sdata = SData::new(cache, self.ntops1, self.ntops2, self.ecut);

    // parse gamestate
    let mut state = State::new(&wbank, self.turns, self.hard);
    let mut gw: Option<Word> = None;
    let mut turn = 0u32;
    let mut it = self.gamestate.split('.');
    while let Some(s_a) = it.next() {
      if s_a.is_empty() {
        break;
      }

      if let Some(s_b) = it.next() {
        // if next guess and feedback exist, follow
        let gw = Word::from_str(s_a).unwrap();
        let fb = Feedback::from_str(s_b).unwrap();
        state = state.fb_follow(gw, fb);
      } else {
        // else set guess word
        gw = Some(Word::from_str(s_a).unwrap());
      }
      turn += 1;
    }

    (state, sdata, gw, turn)
  }

  fn run_glist(&self, state: &State, sdata: &SData) -> Option<DTree> {
    let style = Style::modern()
      .off_horizontal()
      .lines([(1, Style::modern().get_horizontal())]);

    let tops = state.top_words(sdata);
    let mut solutions = tops.into_par_iter()
      .map(|w| (w, state.solve_given(w, sdata, u32::MAX)))
      .collect::<Vec<(Word, Option<DTree>)>>();
    solutions.sort_by_cached_key(|(w, odt)| odt.as_ref().map_or(u32::MAX, |dt| dt.get_tot()));

    println!("Guesses:");
    let gresults = solutions.par_iter().enumerate()
      .map(|(i, (w, odt))| GResult {
        i: i+1,
        guess: *w,
        total: odt.as_ref().map_or(u32::MAX, |odt| odt.get_tot()),
        alen: odt.as_ref().map_or(u32::MAX, |odt| odt.get_alen()),
        eval: odt.as_ref().map_or(f64::INFINITY, |odt| odt.get_eval()),
    }).collect::<Vec<GResult>>();
    println!("{}", Table::new(gresults).with(style));
    println!();
    solutions.remove(0).1
  }

  pub fn run(&self) {
    // define table style
    let style = Style::modern()
      .off_horizontal()
      .lines([(1, Style::modern().get_horizontal())]);

    let (state, sdata, word, turn) = self.load_and_parse();

    // list answers
    if self.alist {
      println!("Answers:");
      let aresults = state.aws.iter().enumerate()
        .map(|(i, answer)| AResult {i: i+1, answer: *answer})
        .collect::<Vec<AResult>>();
      println!("{}", Table::new(aresults).with(style.clone()));
      println!();
    }

    // solve and try to list guesses
    let start = Instant::now();
    let dtree = if !word.is_some() && self.glist {
      self.run_glist(&state, &sdata)
    } else if !word.is_some() {
      state.solve(&sdata, u32::MAX)
    } else {
      state.solve_given(word.unwrap(), &sdata, u32::MAX)
    }
    .expect("couldn't make dtree!");
    let duration = start.elapsed();

    // list feedbacks
    if self.flist {
      if let DTree::Node {tot: _, word: _, ref fbmap} = dtree {
        println!("Feedbacks:");
        let fresults = fbmap.iter().enumerate()
          .map(|(i, (fb, dt))| FResult {
            i: i+1,
            feedback: *fb,
            total: dt.get_tot(),
            alen: dt.get_alen(),
            eval: dt.get_eval(),
          }).collect::<Vec<FResult>>();
        println!("{}", Table::new(fresults).with(style.clone()));
        println!();
      }
    }

    // print solution
    println!("Solution:");
    if let DTree::Node {tot, word, ref fbmap} = dtree {
      println!(
        "{}: {}/{} = {:.4} in {}s",
        word,
        tot,
        dtree.get_alen(),
        dtree.get_eval(),
        duration.as_millis() as f64 / 1000.
      );
    } else {
      println!("No solution, state is a leaf");
    }

    // output dtree to file
    if let Some(dt) = &self.dt {
      let mut f = File::create(dt).unwrap();
      dtree.pprint(&mut f, &"".into(), turn);
    }
  }
}
