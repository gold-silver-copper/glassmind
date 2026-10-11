//! Independent source accounting for the public pilot audit, not a query engine.
use rusqlite::{Connection, OpenFlags};
use std::{path::Path, time::Instant};

// Token separators cannot affect the two-byte eligibility boundary: a name
// needs two ASCII alphanumeric bytes or any multi-byte alphanumeric character
// after lowercasing. Stop at that boundary without constructing normalized keys.
fn eligible(name: &str) -> bool {
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .map(char::len_utf8)
        .take(2)
        .sum::<usize>()
        >= 2
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let [_, source] = args.as_slice() else {
        return Err("usage: pilot_counts PILOT.sqlite".into());
    };
    let started = Instant::now();
    let db = Connection::open_with_flags(
        Path::new(source).canonicalize()?,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    db.execute_batch("PRAGMA query_only=ON; PRAGMA trusted_schema=OFF; PRAGMA cache_size=-65536; PRAGMA mmap_size=0; PRAGMA temp_store=FILE;")?;
    let mut statement = db.prepare("SELECT name,count(*) FROM names GROUP BY entity,name")?;
    let mut rows = statement.query([])?;
    let mut groups = 0u64;
    let mut source_rows = 0u64;
    let mut aliases = 0u64;
    let mut eligible_rows = 0u64;
    while let Some(row) = rows.next()? {
        let name: String = row.get(0)?;
        let occurrences = u64::try_from(row.get::<_, i64>(1)?)?;
        groups += 1;
        source_rows = source_rows
            .checked_add(occurrences)
            .ok_or("source count overflow")?;
        if eligible(&name) {
            aliases += 1;
            eligible_rows = eligible_rows
                .checked_add(occurrences)
                .ok_or("eligible count overflow")?;
        }
    }
    println!(
        "{}",
        serde_json::json!({"source_rows":source_rows,"distinct_original_aliases":groups,
        "eligible_aliases":aliases,"eligible_name_rows":eligible_rows,
        "excluded_name_rows":source_rows-eligible_rows,"seconds":started.elapsed().as_secs_f64()})
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::eligible;
    #[test]
    fn boundary_matches_whole_string_normalization() {
        // Include lowercasing expansions, context-sensitive sigma, combining
        // alphabetic marks, separators and every Unicode scalar by itself.
        let reference = |s: &str| {
            s.to_lowercase()
                .split(|c: char| !c.is_alphanumeric())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
                .len()
                >= 2
        };
        for value in 0..=0x10ffff {
            if let Some(c) = char::from_u32(value) {
                let name = c.to_string();
                assert_eq!(eligible(&name), reference(&name));
            }
        }
        for name in [
            "İ", "I\u{307}", "ΟΣ", "a\u{345}", "a ! b", "!A!", "東京", "", "x\0y",
        ] {
            assert_eq!(eligible(name), reference(name), "{name:?}");
        }
    }
}
