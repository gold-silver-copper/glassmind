use std::io::{self, Write};
fn main() -> io::Result<()> {
    eprintln!("Rust Unicode {:?}", char::UNICODE_VERSION);
    let mut out = io::BufWriter::new(io::stdout().lock());
    for value in 0..=0x10ffff {
        let Some(c) = char::from_u32(value) else { continue; };
        let text = format!("A{c}B");
        let normalized = text.to_lowercase().split(|c: char| !c.is_alphanumeric())
            .filter(|word| !word.is_empty()).collect::<Vec<_>>().join(" ");
        writeln!(out, "{value}\t{normalized}")?;
    }
    Ok(())
}
