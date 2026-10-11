use std::io::{self, BufRead};
const MAX_TOKENS: usize = 128;
type Result<T> = std::result::Result<T, String>;
include!("core.rs");
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
 for line in io::stdin().lock().lines() {
  let line=line?;let mut values=line.split_whitespace();
  let n:usize=values.next().ok_or("no size")?.parse()?;
  let mut scores=vec![vec![None;n+1];n+1];
  for (h,row) in scores.iter_mut().enumerate() {
   for (d,cell) in row.iter_mut().enumerate() {
    let score:f64=values.next().ok_or("short matrix")?.parse()?;
    if h!=d && d!=0 {*cell=Some(score);}
   }
  }
  if values.next().is_some(){return Err("trailing scores".into());}
  let tree=decode(&scores)?;
  println!("{}",tree.iter().map(usize::to_string).collect::<Vec<_>>().join(" "));
 }
 Ok(())
}
