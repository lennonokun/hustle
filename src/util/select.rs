use std::cmp::{min,max};

// floyd-rivest selection algorithm
pub fn select<T>(vec: &mut Vec<T>, k: usize, mut left: usize, mut right: usize)
where T: PartialOrd + Copy {
  // edge cases
  if vec.is_empty() {
    return
  } else if k == 0 {
    select_zero(vec, left, right);
    return
  }

  while right > left {
    if right - left > 600 {
      let n = (right - left + 1) as f32;
      let m = (k - left + 1) as f32;
      let z = n.ln();
      let s = 0.5 * (2.0 * z / 3.0).exp();
      let sign = if (m - n / 2.0) < 0.0 {-1.0} else {1.0};
      let sd = 0.5 * sign * (z * s * (n-s)/n).sqrt();
      let left2 = max(left, (k as f32 - m * s/n + sd).floor() as usize);
      let right2 = min(right, (k as f32 + (n - m) * s/n + sd).floor() as usize);
      select(vec, k, left2, right2)
    }

    let t = vec[k].clone();
    let mut i = left;
    let mut j = right;
    vec.swap(left, k);
    if vec[right] > t {
      vec.swap(left, right);
    }
    while i < j {
      vec.swap(i, j);
      i += 1;
      j -= 1;
      while vec[i] < t {i+=1};
      while vec[j] > t {j-=1};
    }
    if vec[left] == t {
      vec.swap(left, j);
    } else {
      j += 1;
      vec.swap(j, right);
    }

    if j <= k {
      left = j + 1;
    }
    if k <= j {
      right = j - 1;
    }
  }
}

// select for k=0 (just find smallest)
pub fn select_zero<T>(vec: &mut Vec<T>, mut left: usize, mut right: usize)
where T: PartialOrd + Copy {
  let mut j = 0;
  let mut a = vec[0];
  for (i, b) in vec.iter().enumerate().skip(1) {
    if *b < a {
      j = i;
      a = b.clone();
    }
  }

  vec.swap(0, j);
}
#[cfg(test)]
mod test {
  use super::*;
  use rand::prelude::*;

  #[test]
  fn test_select() {
    let mut rng = rand::thread_rng();
    for i in 0..500 {
      let mut vec = (0..100).into_iter().collect::<Vec<usize>>();
      vec.shuffle(&mut rng);
      let k = rand::random::<usize>() % 100;
      println!("{vec:?}, {k}");
      select(&mut vec, k, 0, 99);
      assert_eq!(vec[k], k);
      for i in 0..k {
        assert!(vec[i] < k);
      }
      for i in (k+1)..20 {
        assert!(vec[i] > k);
      }
    }
  }
}
