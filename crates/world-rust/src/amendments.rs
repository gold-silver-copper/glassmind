//! Typed parameter corrections; native proposal checks remain authoritative.
use crate::Rust;
use glassmind_meaning::{Action, Name, Trouble};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RustAmendment {
    RenameDestination(Name),
    FieldDefault(String),
    FieldType(String),
    FieldName(Name),
}

pub(crate) fn revise(
    action: &Action<Rust>,
    amendment: &RustAmendment,
) -> Result<Action<Rust>, Trouble<Rust>> {
    let mut revised = action.clone();
    match (&mut revised, amendment) {
        (Action::Rename { name, .. }, RustAmendment::RenameDestination(destination)) => {
            *name = Some(destination.clone())
        }
        (Action::AddMember { member, .. }, RustAmendment::FieldDefault(value)) => {
            member.default = Some(value.clone())
        }
        (Action::AddMember { member, .. }, RustAmendment::FieldType(ty)) => member.ty = ty.clone(),
        (Action::AddMember { member, .. }, RustAmendment::FieldName(name)) => {
            member.name = name.0.clone()
        }
        _ => {
            return Err(Trouble::ActionFailed {
                reason: "this parameter does not belong to the previous action".into(),
            });
        }
    }
    Ok(revised)
}
