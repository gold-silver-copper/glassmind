type Int = i32;
pub fn pick(items: &[i32], rank: Int) -> Option<i32> {
 if rank < 1 || rank as usize > items.len() { return None; }
 let index = rank - 1;
 Some(items[index])
}
