use std::fs::File;
use std::io::{BufRead, BufReader, Write, Result};
use std::path::Path;

use crate::ds::{NWORDS, NGUESSES};

// loaded heuristic data
// let ht[0] be zero to make indexing easier, so +1
// does this really need to be f64
#[derive(Debug)]
pub struct HData {
	approx: [f64; NWORDS+1],
}

// records heuristic data 
// let ht[0] be zero to make indexing easier, so +1
pub struct HRec {
	// recorded data
	rsums: [[f64; NWORDS+1]; NGUESSES+1],
	rcts: [[i64; NWORDS+1]; NGUESSES+1],
	rinfs: [i64; NGUESSES+1]
}

impl HData {
	pub fn load<P>(p: P) -> Result<Self> where P: AsRef<Path> {
		let file = File::open(p)?;
		let reader = BufReader::new(file);
		let approx = reader.lines()
			.filter_map(|s| s.ok()?.parse::<f64>().ok())
			.collect::<Vec<f64>>()
			.try_into().expect("expected NWORDS+1 lines in heuristic cache");
		Ok(Self {approx})
	}

	#[inline]
	pub fn get_approx(self: &Self, n: usize) -> f64 {
		self.approx[n]
	}
}

impl HRec {
	pub fn new() -> Self {
		Self {
			rsums: [[0.0; NWORDS+1]; NGUESSES+1],
			rcts: [[0; NWORDS+1]; NGUESSES+1],
			rinfs: [0; NGUESSES+1]
		}
	}

	pub fn record(self: &mut Self, m: usize, n: usize, eval: f64) { 
		self.rsums[m][n] += eval;
		self.rcts[m][n] += 1;
	}

	pub fn record_inf(self: &mut Self, n: usize) {
		self.rinfs[n] += 1;
	} 
	// write recorded data to file
	pub fn save(self: &mut Self, path1: &str, path2: &str)
							-> Result<()> {
		let mut out1 = File::create(path1)?;
		writeln!(&mut out1, "m,n,h,ct");
		for m in 1..7 {
			for n in 0..NWORDS+1 {
				let h = if self.rcts[m][n] == 0 {f64::NAN}
				else {self.rsums[m][n] / self.rcts[m][n] as f64};
				writeln!(&mut out1, "{},{},{},{}", m, n, h, self.rcts[m][n]);
			}
		}
		let mut out2 = File::create(path2)?;
		writeln!(&mut out2, "m,ct");
		for m in 1..7 {
			writeln!(&mut out2, "{},{}", m, self.rinfs[m]);
		}
		Ok(())
	}
}
