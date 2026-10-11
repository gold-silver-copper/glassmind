pub fn pick(items: &[i32], mut rank: i32) -> Option<i32> {
 if rank < 1 || rank as usize > items.len() { return None; }
 rank = 0;
 let index = rank - 1;
 Some(items[index])
}
