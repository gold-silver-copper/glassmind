pub fn fetch(values: &[i32], position: i32) -> Option<i32> {
 if position < 1 { return None; }
 if position as usize > values.len() { return None; }
 let index = position - 1;
 Some(values[index])
}
