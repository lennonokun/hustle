use std::fs::File;
use std::io::{BufRead, BufReader, Write, Result};

use crate::ds::NWORDS;

#[derive(Debug)]
pub struct HData {
	// let ht[0] be zero to make indexing easier, so +1
	// loaded data
	approx: [f64; NWORDS+1],
	// recorded data
	rsums: [f64; NWORDS+1],
	rcts: [i64; NWORDS+1],
	rinfs: [i64; 7]
}

impl HData {
	pub fn new() -> Self {
		HData{
			approx:[0.0; NWORDS+1],
			rsums: [0.0; NWORDS+1],
			rcts: [0; NWORDS+1],
			rinfs: [0; 7]
		}
	}

	pub fn record(self: &mut Self, n: usize, eval: f64) { 
		self.rsums[n] += eval;
		self.rcts[n] += 1;
	}

	pub fn record_inf(self: &mut Self, n: usize) {
		self.rinfs[n] += 1;
	} 

	#[inline]
	pub fn get_approx(self: &Self, n: usize) -> f64 {
		self.approx[n]
	}
	
	// read approximated data from file
	pub fn read(self: &mut Self, p: &str) -> Result<()> {
		let file = File::open(p)?;
		let reader = BufReader::new(file);
		self.approx = reader.lines()
			.filter_map(|s| s.ok()?.parse::<f64>().ok())
			.collect::<Vec<f64>>()
			.try_into().expect("expected NWORDS+1 lines in heuristic cache");
		Ok(())
	}

	// write recorded data to file
	pub fn write(self: &mut Self, path: &str) -> Result<()> {
		let mut out = File::create(path)?;
		writeln!(&mut out, "x,y,ct");
		for i in 0..NWORDS+1 {
			let y = if self.rcts[i] == 0 {0.0}
			else {self.rsums[i] / self.rcts[i] as f64};

			writeln!(&mut out, "{},{},{}", i, y, self.rcts[i]);
		}
		Ok(())
	}
}

// pub fn write_data(path: &str, data: &Vec<(f64, f64)>) -> Result<()> {
// }

