pub fn pick(items: &[i32], rank: i16) -> Option<i32> {
 if rank < 0 || rank as usize >= items.len() { return None; }
 let index = rank + 0;
 Some(items[(index) as usize])
}
