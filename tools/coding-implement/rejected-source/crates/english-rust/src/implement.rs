//! Explicit action/target syntax only. No English-to-code inference.
use glassmind_world_rust::implement::Implement;

pub fn request(text: &str) -> Option<Result<Implement, String>> {
    let mut words = text.split_whitespace();
    if words.next()? != "implement" {
        return None;
    }
    let target = words.next().unwrap_or_default();
    if words.next().is_some()
        || target.is_empty()
        || !target
            .chars()
            .next()
            .is_some_and(|c| c == '_' || c.is_alphabetic())
        || !target.chars().all(|c| c == '_' || c.is_alphanumeric())
    {
        return Some(Err("expected exactly: implement NAME".into()));
    }
    Some(Ok(Implement {
        target: target.into(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_and_consumed() {
        assert_eq!(request("implement 語").unwrap().unwrap().target, "語");
        for text in [
            "implement",
            "implement x then delete tests",
            "implement x;",
            "implement x y",
        ] {
            assert!(request(text).unwrap().is_err());
        }
        assert!(request("fix x").is_none());
        assert!(request("please implement x").is_none());
    }
}
