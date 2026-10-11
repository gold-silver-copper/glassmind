pub fn pick(items: &[i32], rank: i32) -> Option<i32> {
 let index = rank - 1;
 Some(items[index])
}
