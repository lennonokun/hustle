
fn partition<T>(vec: &mut Vec<T>, left: usize, right: usize) -> usize
where T: PartialOrd + Clone {
  let pval = vec[(left+right)/2].clone();

  // first iteration is on the house (no negatives)
  let mut i = left;
  let mut j = right;
  while vec[i] < pval {i+=1;}
  while vec[j] > pval {j-=1;}
  if i >= j {return j}
  vec.swap(i, j);

  loop {
    i += 1;
    while vec[i] < pval {i += 1;}
    j -= 1;
    while vec[j] > pval {j -= 1;}
    if i >= j {return j;}
    vec.swap(i, j);
  }
}

pub fn qselect<T>(vec: &mut Vec<T>, k: usize, left: usize, right: usize) 
where T: PartialOrd + Clone {
  if left == right {
    return;
  }

  let pivot = partition(vec, left, right);

  if k == pivot {
    return;
  } else if k < pivot {
    qselect(vec, k, left, pivot-1)
  } else {
    qselect(vec, k, pivot+1, right)
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use rand::prelude::*;

  #[test]
  fn select() {
    let mut vec = (0..100).rev().collect::<Vec<usize>>();
    qselect(&mut vec, 99, 0, 99);

    let mut rng = rand::thread_rng();
    for i in 0..50 {
      let mut vec = (0..100).into_iter().collect::<Vec<usize>>();
      vec.shuffle(&mut rng);
      let k = rand::random::<usize>() % 100;
      qselect(&mut vec, k, 0, 99);
      assert_eq!(vec[k], k);
      for i in 0..k {
        assert!(vec[i] < k);
      }
      for i in k..100 {
        assert!(vec[i] > k);
      }
    }
  }
}
