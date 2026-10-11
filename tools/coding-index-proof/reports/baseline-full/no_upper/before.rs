pub fn pick(items: &[i32], rank: i32) -> Option<i32> {
 if rank < 1 { return None; }
 let index = rank - 1;
 Some(items[index])
}
