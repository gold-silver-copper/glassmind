pub fn pick(items: &Custom, rank: i32) -> Option<i32> {
 if rank < 1 || rank as usize > items.len() { return None; }
 let index = rank - 1;
 Some(items[index])
}
