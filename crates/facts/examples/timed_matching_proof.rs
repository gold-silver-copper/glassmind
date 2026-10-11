//! Parent adapter: typed constraints omit original wording, so compare only date APIs.
use glassmind_facts::temporal::{CalendarDate, TemporalIndex};
use std::io::{self, BufRead};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let [_, index, temporal] = args.as_slice() else {
        return Err("INDEX TEMPORAL".into());
    };
    let loaded = TemporalIndex::from_bytes(&std::fs::read(index)?, &std::fs::read(temporal)?)?;
    for line in io::stdin().lock().lines() {
        let (date, question): (String, String) = serde_json::from_str(&line?)?;
        let answer = loaded.match_at_str(&date, &question);
        if let Ok(date) = CalendarDate::parse(&date)
            && answer != loaded.match_at(&date, &question)
        {
            return Err("date API mismatch".into());
        }
        println!("{}", serde_json::to_string(&answer)?);
    }
    Ok(())
}
