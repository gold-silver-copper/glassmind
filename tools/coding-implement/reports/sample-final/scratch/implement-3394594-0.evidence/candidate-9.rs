pub fn invertir(xs: &[i32]) -> Vec<i32> { return xs.iter().copied().filter(|v| *v > 0).collect::<::std::vec::Vec<_>>() }
// fixture tail is immutable
#[cfg(test)] mod tests { use super::*; #[test] fn contract() { assert_eq!(invertir(&[-3,0,4,7]),vec![7,4,0,-3]); assert_eq!(invertir(&[]),vec![]); assert_eq!(invertir(&[8]),vec![8]); } }
