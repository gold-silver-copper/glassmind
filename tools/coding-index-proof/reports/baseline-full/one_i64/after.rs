pub fn pick(items: &[i32], rank: i64) -> Option<i32> {
 if rank <= 0 || rank as usize > items.len() { return None; }
 let index = rank - 1;
 Some(items[index])
}
