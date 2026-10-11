//! Entity-bound inventories and checked events, independent of surface English.
use crate::rational::{Rational, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entity(pub String);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectKind(pub String);

#[derive(Clone, Copy, Debug)]
pub enum Change {
    Receive(Rational),
    Remove(Rational),
}

#[derive(Clone, Debug)]
pub struct Inventory {
    pub owner: Entity,
    pub kind: ObjectKind,
    count: Rational,
}
impl Inventory {
    pub fn new(owner: Entity, kind: ObjectKind, count: Rational) -> Result<Self> {
        if count.is_negative() {
            return Err("negative inventory".into());
        }
        Ok(Self { owner, kind, count })
    }
    pub fn count(&self) -> Rational {
        self.count
    }
    pub fn apply(&mut self, owner: &Entity, kind: &ObjectKind, change: Change) -> Result<()> {
        if owner != &self.owner || kind != &self.kind {
            return Err("inventory entity or object kind mismatch".into());
        }
        let (Change::Receive(amount) | Change::Remove(amount)) = change;
        if amount.is_negative() {
            return Err("negative event amount".into());
        }
        let next = match change {
            Change::Receive(amount) => self.count.checked_add(amount)?,
            Change::Remove(amount) => self.count.checked_sub(amount)?,
        };
        if next.is_negative() {
            return Err("event removes more than available".into());
        }
        self.count = next;
        Ok(())
    }
    pub fn difference(&self, other: &Self) -> Result<Rational> {
        if self.kind != other.kind {
            return Err("cannot compare different object kinds".into());
        }
        self.count.checked_sub(other.count)
    }
}
