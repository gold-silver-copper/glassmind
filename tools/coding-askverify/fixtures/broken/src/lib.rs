// SPDX-License-Identifier: MIT OR Apache-2.0
pub struct Parcel {
    pub weight: u32,
}
pub fn weigh(parcel: &Parcel) -> u32 {
    parcel.weight
}
pub fn shipment() -> Parcel {
    Parcel { weight: 7 }
}
pub fn broken() -> u32 {
    "not a number"
}
