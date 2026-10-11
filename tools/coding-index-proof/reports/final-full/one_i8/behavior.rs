#[test]
fn independent_contract() {
 let data = [12, 34, 56];
 for n in [i8::MIN, -2, -1, 0, 1, 2, 3, 4, i8::MAX] {
   let expected = match n { 1 => Some(12), 2 => Some(34), 3 => Some(56), _ => None };
   assert_eq!(task::pick(&data, n), expected, "rank {n}");
   assert_eq!(task::pick(&[], n), None);
 }
}
