pub fn pick(items: &[i32], rank: i32) -> Option<i32> {
 if rank < 1 || rank as usize > items.len() { return None; }
 #[allow(unused)] let index = rank - 1;
 Some(items[index])
}
