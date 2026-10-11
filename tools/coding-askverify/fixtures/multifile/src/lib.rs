// SPDX-License-Identifier: MIT OR Apache-2.0
pub mod parcel;
pub fn shipment() -> parcel::Parcel {
    parcel::Parcel { weight: 7 }
}
pub fn broken() -> u32 {
    "not a number"
}
