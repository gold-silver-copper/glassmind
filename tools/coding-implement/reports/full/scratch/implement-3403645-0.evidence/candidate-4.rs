pub fn aggregate(xs: &[i32]) -> i32 { return xs.iter().copied().sum() }
// fixture tail is immutable
#[cfg(test)] mod tests { use super::*; #[test] fn contract() { assert_eq!(aggregate(&[-7,2,4,9]),8); assert_eq!(aggregate(&[-3,-2]),-5); assert!(!std::env::current_dir().unwrap().ends_with("live_rollback")); } }
