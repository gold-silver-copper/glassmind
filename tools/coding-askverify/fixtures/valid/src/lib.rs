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
#[cfg(test)]
mod tests {
    #[test]
    fn original_behavior() {
        assert_eq!(super::weigh(&super::shipment()), 7);
    }
}
