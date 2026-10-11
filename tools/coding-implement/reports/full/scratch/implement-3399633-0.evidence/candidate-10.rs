pub fn invertir(xs: &[i32]) -> Vec<i32> { return xs.iter().copied().filter(|v| *v > 0).rev().collect::<::std::vec::Vec<_>>() }
// fixture tail is immutable
#[cfg(test)] mod tests { use super::*; #[test] fn contract() { assert_eq!(invertir(&[-3,-2,0,4,7]),vec![-2,0,4]); assert_eq!(invertir(&[]),vec![]); } }
