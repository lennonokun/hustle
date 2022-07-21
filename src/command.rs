use clap::{Parser, Subcommand};
use super::ds::{DEFWBP, DEFHDP};

#[derive(Parser)]
#[clap(version, about)]
pub struct Cli {
  #[clap(subcommand)]
  pub command: Commands,
}
 
#[derive(Subcommand)]
pub enum Commands {
  /// play hustle
  Play,
  /// solve game state
  Solve {
    /// the game state to solve from
    #[clap(value_parser, default_value = "")]
    gamestate: String,
    /// list top word evaluations
    #[clap(long)]
    elist: bool,
    /// list potential answers
    #[clap(long)]
    alist: bool,
    /// output decision tree to file
    #[clap(long)]
    dt: Option<String>,
    /// word length
    #[clap(long, default_value_t = 5)]
    wlen: u8,
    /// word bank path
    #[clap(long, default_value_t=String::from(DEFWBP))]
    wbp: String,
    /// heuristic data path
    #[clap(long, default_value_t=String::from(DEFHDP))]
    hdp: String,
    /// play in hard mode
    #[clap(long)]
    hard: bool,
    /// the number of top words to check at each state
    #[clap(long, default_value_t = 10)]
    ntops: u32,
    /// the maximum number of answer words left for an "endgame"
    #[clap(long, default_value_t = 15)]
    ecut: u32,
    /// the minimum number of answers word left to cache
    #[clap(long, default_value_t = 30)]
    ccut: u32,
  },
  /// generate heuristic data
  Hgen {
    /// the number of data points to generate
    #[clap(value_parser)]
    niter: usize,
    /// the file to output data to
    #[clap(value_parser)]
    out: String,
    /// word length
    #[clap(long, default_value_t = 5)]
    wlen: u8,
    /// word bank path
    #[clap(long, default_value_t=String::from(DEFWBP))]
    wbp: String,
    /// heuristic data path
    #[clap(long, default_value_t=String::from(DEFHDP))]
    hdp: String,
    /// the number of 
    #[clap(long, default_value_t = 3)]
    ntops: usize,
    /// endgame cutoff
    #[clap(long, default_value_t = 15)]
    ecut: u32,
    /// cache cutoff
    #[clap(long, default_value_t = 30)]
    ccut: u32,
  },
//  /// generate general data
//  Ggen {
//    /// the number of data points to generate
//    #[clap(value_parser)]
//    niter: usize,
//    /// the file to output data to
//    #[clap(value_parser)]
//    out: String,
//    /// word length
//    #[clap(long, default_value_t = 5)]
//    wlen: u8,
//    /// word bank path
//    #[clap(long, default_value_t=String::from(DEFWBP))]
//    wbp: String,
//    /// heuristic data path
//    #[clap(long, default_value_t=String::from(DEFHDP))]
//    hdp: String,
//    /// the range of ntops to try
//    #[clap(long)]
//    ntops: Option<String>,
//    /// endgame cutoff
//    #[clap(long)]
//    ecut: Option<String>,
//    /// cache cutoff
//    #[clap(long)]
//    ccut: Option<String>,
//  },
}

pub fn cli_parse() -> Cli {
  Cli::parse()
}
