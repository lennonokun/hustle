use clap::{Parser, Subcommand};

use crate::util::{Range, DEFWBP, DEFHDP, DEFLDP};

#[derive(Parser)]
#[clap(version, about)]
pub struct Cli {
  #[clap(subcommand)]
  pub command: Commands,
}
 
#[derive(Subcommand)]
pub enum Commands {
  /// play hustle
  #[cfg(feature = "play")]
  Play,
  /// solve game state
  #[cfg(feature = "solve")]
  Solve {
    /// the game state to solve from
    #[clap(value_parser, default_value="")]
    gamestate: String,
    /// list potential answers
    #[clap(long, short='a')]
    alist: bool,
    /// list potential feedbacks and their evaluations
    #[clap(long, short='f')]
    flist: bool,
    /// list top guess evaluations
    #[clap(long, short='g')]
    glist: bool,
    /// output decision tree to file
    #[clap(long, short='o')]
    dt: Option<String>,
    /// word length
    #[clap(long, short='l', default_value_t=5)]
    wlen: u8,
    /// word bank path
    #[clap(long, short='B', default_value_t=String::from(DEFWBP))]
    wbp: String,
    /// heuristic data path
    #[clap(long, short='H', default_value_t=String::from(DEFHDP))]
    hdp: String,
    /// lower bounds data path
    #[clap(long, short='L', default_value_t=String::from(DEFLDP))]
    ldp: String,
    /// play in hard mode
    #[clap(long)]
    hard: bool,
    /// the number of rows/sets in the cache
    #[clap(long, short='M', default_value_t=64)]
    ncacherows: usize,
    /// the number of columns in the cache
    #[clap(long, short='N', default_value_t=16)]
    ncachecols: usize,
    /// the number of top soft heuristic words to try
    #[clap(long, short='m', default_value_t=1000)]
    ntops1: u32,
    /// the number of top hard heuristic words to try
    #[clap(long, short='n', default_value_t=10)]
    ntops2: u32,
    /// the maximum number of turns to solve in
    #[clap(long, short='t', default_value_t=6)]
    turns: u32,
    /// the maximum number of answer words left for an "endgame"
    #[clap(long, short='e', default_value_t=15)]
    ecut: u32,
  },
  /// generate heuristic data
  #[cfg(feature = "gen")]
  Hgen {
    /// the number of data points to generate
    #[clap(value_parser)]
    niter: usize,
    /// the file to output data to
    #[clap(value_parser)]
    out: String,
    /// word length
    #[clap(long, short='l', default_value_t=5)]
    wlen: u8,
    /// word bank path
    #[clap(long, short='B', default_value_t=String::from(DEFWBP))]
    wbp: String,
    /// heuristic data path
    #[clap(long, short='H', default_value_t=String::from(DEFHDP))]
    hdp: String,
    /// lower bounds data path
    #[clap(long, short='L', default_value_t=String::from(DEFLDP))]
    ldp: String,
    /// the number of rows/sets in the cache
    #[clap(long, short='M', default_value_t=64)]
    ncacherows: usize,
    /// the number of columns in the cache
    #[clap(long, short='N', default_value_t=16)]
    ncachecols: usize,
    /// the number of top soft heuristic words to try
    #[clap(long, short='m', default_value_t=500)]
    ntops1: u32,
    /// the number of top hard heuristic words to try
    #[clap(long, short='n', default_value_t=5)]
    ntops2: u32,
    /// the maximum number of turns to solve in
    #[clap(long, short='t', default_value_t=6)]
    turns: u32,
    /// endgame cutoff
    #[clap(long, short='e', default_value_t=15)]
    ecut: u32,
  },
  /// generate general data
  #[cfg(feature = "gen")]
  Ggen {
    /// the number of data points to generate
    #[clap(value_parser)]
    niter: usize,
    /// the file to output data to
    #[clap(value_parser)]
    out: String,
    /// word length
    #[clap(long, short='l', default_value_t = 5)]
    wlen: u8,
    /// word bank path
    #[clap(long, short='B', default_value_t=String::from(DEFWBP))]
    wbp: String,
    /// heuristic data path
    #[clap(long, short='H', default_value_t=String::from(DEFHDP))]
    hdp: String,
    /// lower bounds data path
    #[clap(long, short='L', default_value_t=String::from(DEFLDP))]
    ldp: String,
    /// the range of answer lengths to try (defaults to all)
    #[clap(long, short='a')]
    alens: Option<Range<usize>>,
    // TODO: make range (only power of 2)?
    /// the number of rows/sets in the cache
    #[clap(long, short='M', default_value_t=64)]
    ncacherows: usize,
    // TODO: make range (only power of 2)?
    /// the number of columns in the cache
    #[clap(long, short='N', default_value_t=16)]
    ncachecols: usize,
    /// the range of number of top soft heuristic words to try
    #[clap(long, short='m', default_value_t=Range::new(1, 1000, true))]
    ntops1: Range<u32>,
    /// the range of number of top hard heuristic words to try
    #[clap(long, short='n', default_value_t=Range::new(1, 10, true))]
    ntops2: Range<u32>,
    /// the range of maximum numbers of turns to solve in
    #[clap(long, short='t', default_value_t=Range::new(1, 6, true))]
    turns: Range<u32>,
    /// endgame cutoff
    #[clap(long, short='e', default_value_t=Range::new(1, 30, true))]
    ecut: Range<u32>,
  },
  /// generate lower bounds data
  #[cfg(feature = "gen")]
  Lgen {
    /// the number of tries at each alen
    #[clap(value_parser)]
    niter: usize,
    /// the step between each alen to try
    #[clap(value_parser)]
    step: usize,
    /// the file to output data to
    #[clap(value_parser)]
    out: String,
    /// word length
    #[clap(long, short='l', default_value_t = 5)]
    wlen: u8,
    /// word bank path
    #[clap(long, short='B', default_value_t=String::from(DEFWBP))]
    wbp: String,
    /// lower bounds data path
    #[clap(long, short='L', default_value_t=String::from(DEFHDP))]
    ldp: String,
    /// heuristic data path
    #[clap(long, short='H', default_value_t=String::from(DEFLDP))]
    hdp: String,
    /// the range of answer lengths to try
    #[clap(long, short='a')]
    alens: Option<Range<usize>>,
    /// the number of rows/sets in the cache
    #[clap(long, short='M', default_value_t=64)]
    ncacherows: usize,
    /// the number of columns in the cache
    #[clap(long, short='N', default_value_t=16)]
    ncachecols: usize,
    /// the number of top soft heuristic words to try
    #[clap(long, short='m', default_value_t=1000)]
    ntops1: u32,
    /// the number of top hard heuristic words to try
    #[clap(long, short='n', default_value_t=10)]
    ntops2: u32,
    /// the maximum number of turns to solve in
    #[clap(long, short='t', default_value_t=6)]
    turns: u32,
    /// endgame cutoff
    #[clap(long, short='e', default_value_t=15)]
    ecut: u32,
  },
}

pub fn cli_parse() -> Cli {
  Cli::parse()
}
