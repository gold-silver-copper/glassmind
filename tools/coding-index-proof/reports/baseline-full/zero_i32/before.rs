pub fn pick(items: &[i32], rank: i32) -> Option<i32> {
 if rank < 0 || rank as usize >= items.len() { return None; }
 
 Some(items[rank])
}
