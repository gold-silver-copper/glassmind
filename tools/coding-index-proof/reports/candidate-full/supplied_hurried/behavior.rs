#[test] fn independent_contract() {
 let data = ["Ada", "Bea", "Cy"];
 for n in [i32::MIN, -2, -1, 0, 1, 2, 3, 4, i32::MAX] {
  let expected = match n { 1 => Some("Ada".to_string()), 2 => Some("Bea".to_string()), 3 => Some("Cy".to_string()), _ => None };
  assert_eq!(task::player_at_rank(&data,n),expected);
  assert_eq!(task::player_at_rank(&[],n),None);
 }
}