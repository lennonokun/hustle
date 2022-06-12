use std::fs::File;
use std::io::{BufRead, BufReader, Write, Result};

use crate::ds::NWORDS;

pub type HCache = [f64; NWORDS+1];

// pub fn write_data(path: &str, data: &Vec<(f64, f64)>) -> Result<()> {
	// let mut out = File::create(path)?;
	// writeln!(&mut out, "x,y");
	// for (x,y) in data {
		// writeln!(&mut out, "{},{}", x, y);
	// }
	// Ok(())
// }

// make heuristic cache from file
pub fn read_heuristic(p: &str) -> Result<HCache> {
	let file = File::open(p)?;
	let reader = BufReader::new(file);
	Ok(reader.lines()
		 .filter_map(|s| s.ok()?.parse::<f64>().ok())
		 .collect::<Vec<f64>>()
		 .try_into().expect("expected 2316 lines in heuristic cache"))
}
