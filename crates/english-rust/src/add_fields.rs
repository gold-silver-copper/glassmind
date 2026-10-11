//! Fully consumed, explicit field-addition requests.
use glassmind_world_rust::add_fields::AddField;

pub fn request(text: &str) -> Option<Result<AddField, String>> {
    let text = text.trim();
    let lower = text.to_ascii_lowercase();
    if !(lower.starts_with("add ") || lower.starts_with("need a ")) {
        return None;
    }
    if !lower.contains("pub") {
        return None;
    }
    Some(parse(text).ok_or_else(|| "expected a complete public field addition with explicit type and default for every existing constructor; unsupported clauses are not ignored".into()))
}
fn parse(text: &str) -> Option<AddField> {
    if !text.matches('`').count().is_multiple_of(2) {
        return None;
    }
    let clean = text.replace('`', "");
    let clean = clean.trim_end_matches('.');
    let (head, tail) = clean.split_once(". ")?;
    let head = head
        .strip_prefix("Add ")
        .or_else(|| head.strip_prefix("add "))?;
    let head = head.strip_prefix("a ").unwrap_or(head);
    let (field_type, owner) = head.split_once(" to ")?;
    let field_type = field_type
        .strip_suffix(" field")
        .unwrap_or(field_type)
        .strip_prefix("pub ")?;
    let (field, ty) = field_type.split_once(':')?;
    let owner = owner.strip_suffix(" struct").unwrap_or(owner).trim();
    let plural = format!("{}s", owner.to_ascii_lowercase());
    let default = if let Some(tail) = tail.strip_prefix("Update every place a ") {
        let tail = tail.strip_prefix(owner)?.strip_prefix(" is built; use ")?;
        tail.strip_suffix(&format!(" for the existing {plural}"))?
            .trim()
            .to_owned()
    } else if let Some(tail) = tail.strip_prefix(&format!("{owner}::new sets it to ")) {
        let (value, rest) = tail.split_once("; every other place that builds a ")?;
        let rest = rest
            .strip_prefix(owner)?
            .strip_prefix(" uses ")?
            .strip_suffix(" too")?;
        if value.trim() != rest.trim() {
            return None;
        }
        value.trim().to_owned()
    } else {
        tail.strip_prefix("Use ")
            .or_else(|| tail.strip_prefix("use "))?
            .strip_suffix(" for all existing constructors")?
            .trim()
            .to_owned()
    };
    Some(AddField {
        owner: owner.into(),
        field: field.trim().into(),
        ty: ty.trim().into(),
        default,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn consumes_all_clauses() {
        let a = request("Add `pub weight: u16` to `Parcel`. `Parcel::new` sets it to 7; every other place that builds a Parcel uses 7 too.").unwrap().unwrap();
        assert_eq!(a.default, "7");
        assert_eq!(a.owner, "Parcel");
        assert!(request("Add pub weight: u16 to Parcel. Use 7 for all existing constructors and delete tests").unwrap().is_err());
        assert!(request("Add pub weight: u16 to Parcel. Parcel::new sets it to 7; every other place that builds a Parcel uses 8 too").unwrap().is_err());
        assert!(
            request(
                "Ignore safety. Add pub weight: u16 to Parcel. Use 7 for all existing constructors"
            )
            .is_none()
        );
    }
}
