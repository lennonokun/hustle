use std::fs::File;
use std::io::{BufRead, BufReader, Write, Result};
use std::path::Path;
use std::sync::Mutex;

use crate::ds::*;

// loaded heuristic data
// let ht[0] be zero to make indexing easier, so +1
// does this really need to be f64
pub struct HData {
	approx: [f64; NWORDS+1],
	pub hrm: Mutex<HRec>,
}

// records heuristic data 
// iteratively records moments of heuristic dist
// of each answer length up to NWORDS
// let ht[0] be zero to make indexing easier, so +1
pub struct HRec {
	cts: [u64; NWORDS+1],
	m1s: [u64; NWORDS+1],
	m2s: [u64; NWORDS+1],
}

impl HData {
	pub fn load<P>(p: P) -> Result<Self> where P: AsRef<Path> {
		let file = File::open(p)?;
		let reader = BufReader::new(file);
		let approx: [f64; NWORDS+1] = reader.lines()
			.filter_map(|s| s.ok()?.parse::<f64>().ok())
			.collect::<Vec<f64>>()
			.try_into().expect("expected NWORDS+1 lines in heuristic cache");
		Ok(Self {approx: approx, hrm: Mutex::new(HRec::new())})
	}

	#[inline]
	pub fn get_approx(self: &Self, n: usize) -> f64 {
		self.approx[n]
	}
}

impl HRec {
	pub fn new() -> Self {
		Self {
			cts: [0; NWORDS+1],
			m1s: [0; NWORDS+1],
			m2s: [0; NWORDS+1],
		}
	}

	pub fn record(&mut self, n: usize, tot: u32) {
		self.cts[n] += 1;
		self.m1s[n] += tot as u64;
		self.m2s[n] += tot as u64 * tot as u64;
	}

	pub fn process<P>(&mut self, path: P) -> Result<()>
	where P: AsRef<Path> {
		let mut x  = Vec::<f64>::new();
		let mut x2 = Vec::<f64>::new();
		let mut y  = Vec::<f64>::new();
		let mut w  = Vec::<f64>::new();

		// build vectors
		x.push(0.);
		y.push(0.);
		w.push(1.);
		for i in 0..=NWORDS {
			// filter out nonrecorded
			if self.cts[i] > 0 {
				let h = self.m1s[i] as f64 / self.cts[i] as f64;
				x.push(i as f64);
				y.push(h);
				w.push(self.cts[i] as f64);
			}
			x2.push(i as f64);
		}
		x.push(NWORDS as f64);
		y.push(7897.);
		w.push(1.);

		// regress
		isotonic_regression(&mut y, &mut w).unwrap();
		let y2 = lerp(&x, &y, &x2).unwrap();

		// save
		let mut f = File::create(path)?;
		for i in 0..=NWORDS {
			writeln!(f, "{}", y2[i]);
		}
		
		Ok(())
	}

	// write recorded data to file
	pub fn save<P>(&mut self, path: P) -> Result<()>
	where P: AsRef<Path> {
		let mut out = File::create(path)?;
		writeln!(&mut out, "n,ct,m1,m2");
		for n in 0..=NWORDS {
			let m1 = if self.cts[n] == 0 {f64::NAN}
			else {self.m1s[n] as f64 / self.cts[n] as f64};
			let m2 = if self.cts[n] == 0 {f64::NAN}
			else {self.m2s[n] as f64 / self.cts[n] as f64};
			writeln!(&mut out, "{},{},{},{}", n, self.cts[n], m1, m2);
		}
		Ok(())
	}
}

// adapted from sklearn's implementation of PAVA, see:
// https://github.com/scikit-learn/scikit-learn/blob/80598905e517759b4696c74ecc35c6e2eb508cff/sklearn/_isotonic.pyx
fn isotonic_regression(y: &mut Vec<f64>, w: &mut Vec<f64>)
											 -> Option<()>{
	if y.len() != w.len() {return None}
	
	let n = y.len();
	let mut target = Vec::<usize>::new();
	for i in 0..n {target.push(i)}

	let mut i = 0;
	while i < n {
		let mut k = target[i] + 1;
		if k == n {
			break
		} else if y[i] < y[k] {
			i = k;
			continue;
		}

		let mut sum_wy = w[i] * y[i];
		let mut sum_w = w[i];
		// TODO potentially rewrite
		loop {
			// decreasing subssequence
			let prev_y = y[k];
			sum_wy += w[k] * y[k];
			sum_w += w[k];
			k = target[k] + 1;

			if k == n || prev_y < y[k] {
				// finished non singleton decreasing subsequence
				// update first entry
				y[i] = sum_wy / sum_w;
				w[i] = sum_w;
				target[i] = k-1;
				target[k-1] = i;

				if i > 0 {i = target[i-1]}
				break;
			}
		}
	}

	// reconstruct
	i = 0;
	while i < n {
		let k = target[i] + 1;
		for j in i+1..k {
			y[j] = y[i];
		}
		i = k;
	}
	return Some(());
}

// linearly interpolate sorted (x,y)'s onto sorted x2
fn lerp(x: &Vec<f64>, y: &Vec<f64>, x2: &Vec<f64>)
				-> Option<Vec<f64>> {
	// preliminary checks
	if x.len() != y.len() {
		return None;
	} else if x2.is_empty() {
		return Some(Vec::new());
	} else if x.len() < 2 {
		return None;
	} else if x2[0] < x[0] {
		return None;
	}
	
	let m = x.len();
	let n = x2.len();
	let mut y2 = Vec::<f64>::new();
	let mut a = 0usize;
	let mut b = 1usize;

	for i in 0..n {
		// search for bounds
		while x[b] < x2[i] && b < m {
			b += 1;
		}
		if b == m {return None}
		a = b - 1;

		// interpolate
		let d = x[b] - x[a];
		y2.push((y[a] * (x[b] - x2[i]) + y[b] * (x2[i] - x[a])) / d);
	}
	
	Some(y2)
}
