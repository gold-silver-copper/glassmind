pub fn sum(xs: &[i32]) -> i32 { return xs.iter().copied().sum() }
#[cfg(test)] mod tests { use super::*; #[test] fn check() { assert_eq!(sum(&[-7,2,4,9]),8); assert_eq!(sum(&[-3,-2]),-5); } }
