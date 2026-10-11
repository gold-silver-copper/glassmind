pub fn pick(items: &[i32], rank: i16) -> Option<i32> {
 if 1 > rank || rank as usize > items.len() { return None; }
 let index = rank - 1;
 Some(items[index])
}
