//! Fully consumed corrections of a single explicitly named action parameter.
use super::{Cursor, tokens};
use glassmind_meaning::Name;
use glassmind_world_rust::amendments::RustAmendment;

pub fn request(text: &str) -> Option<Result<RustAmendment, String>> {
    if text.len() > 8192 {
        return Some(Err("parameter correction exceeds 8192 bytes".into()));
    }
    let tokens = tokens(text)?;
    let mut c = Cursor {
        source: text,
        tokens: &tokens,
        at: 0,
        owner_previous: false,
    };
    c.eat("actually");
    c.eat(",");
    c.eat("please");
    let begin = c.clone();
    let result = if c.eat("change the default to") || c.eat("set the default to") {
        c.literal().map(RustAmendment::FieldDefault)
    } else if c.eat("change the field type to") || c.eat("set the field type to") {
        c.id().map(RustAmendment::FieldType)
    } else if c.eat("call the field") || c.eat("name the field") {
        c.id().map(|name| RustAmendment::FieldName(Name(name)))
    } else if c.eat("use") {
        let mut value = c.clone();
        if let Some(literal) = value.literal()
            && (value.eat("as the default") || value.eat("for all existing constructors"))
        {
            c = value;
            Some(RustAmendment::FieldDefault(literal))
        } else {
            let name = c.id()?;
            if c.eat("as the field type") {
                Some(RustAmendment::FieldType(name))
            } else if c.eat("as the new name") {
                Some(RustAmendment::RenameDestination(Name(name)))
            } else {
                return None;
            }
        }
    } else if c.eat("call it") {
        let name = c.id()?;
        if !c.eat("instead") {
            return None;
        }
        Some(RustAmendment::RenameDestination(Name(name)))
    } else {
        return None;
    };
    // No discarded suffixes, negations or extra instructions.
    c.eat("instead");
    c.eat("please");
    let _ = c.eat(".") || c.eat("!") || c.eat("?");
    Some(
        result
            .filter(|_| c.at == tokens.len() && c.at > begin.at)
            .ok_or_else(|| {
                "expected one complete parameter correction; unsupported clauses are not ignored"
                    .into()
            }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn corrections_retain_literal_bytes_and_consume_all_words() {
        for (text, expected) in [
            (
                "Actually, change the default to -12.",
                RustAmendment::FieldDefault("-12".into()),
            ),
            (
                "use 'λ' as the default instead",
                RustAmendment::FieldDefault("'λ'".into()),
            ),
            (
                "Use bool as the field type",
                RustAmendment::FieldType("bool".into()),
            ),
            (
                "please name the field active instead",
                RustAmendment::FieldName(Name("active".into())),
            ),
            (
                "call it Shipment instead",
                RustAmendment::RenameDestination(Name("Shipment".into())),
            ),
            (
                "use `Other` as the new name",
                RustAmendment::RenameDestination(Name("Other".into())),
            ),
        ] {
            assert_eq!(request(text).unwrap().unwrap(), expected, "{text}");
        }
        for text in [
            "change the default to 7 and delete tests",
            "use 7 as the default but not everywhere",
            "call it Shipment instead; delete files",
            "change the field type to u8 or bool",
            "name the field one two",
            "change the default to make()",
        ] {
            assert!(request(text).is_some_and(|r| r.is_err()), "{text}");
        }
        assert!(request("call it nickname").is_none());
        assert!(request("where is Parcel?").is_none());
    }
}
