//! What every model's trainer reports: how often the model is right on
//! examples it did not learn from, and whether its confidence means what it
//! says (of the answers it gives with 0.9, are nine in ten right?).

use std::fmt::Write as _;

/// Bands of confidence, from the top.
const BANDS: &[f64] = &[0.99, 0.95, 0.9, 0.8, 0.6, 0.4, 0.0];

/// Held-out accuracy by band of confidence.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Calibration {
    /// For each band (its lowest confidence): how many answers fell in it
    /// and how many of those were right.
    pub bands: Vec<(f64, u32, u32)>,
}

impl Calibration {
    /// From (confidence, right) pairs on held-out examples.
    pub fn of(answers: &[(f64, bool)]) -> Calibration {
        let mut bands: Vec<(f64, u32, u32)> = BANDS.iter().map(|&lo| (lo, 0, 0)).collect();
        for &(p, right) in answers {
            if let Some(band) = bands.iter_mut().find(|b| p >= b.0) {
                band.1 += 1;
                band.2 += u32::from(right);
            }
        }
        Calibration { bands }
    }

    /// Right answers out of all.
    pub fn accuracy(&self) -> (u32, u32) {
        self.bands
            .iter()
            .fold((0, 0), |(r, n), b| (r + b.2, n + b.1))
    }

    /// The table, one band a line, each starting with `prefix` (such as
    /// `# ` to make comment lines of a model file).
    pub fn table(&self, prefix: &str) -> String {
        let mut s = String::new();
        let (right, all) = self.accuracy();
        let _ = writeln!(s, "{prefix}held out: {right} of {all} right");
        for (lo, n, right) in &self.bands {
            if *n > 0 {
                let _ = writeln!(
                    s,
                    "{prefix}  confidence {lo:.2}+: {right}/{n} right ({:.1}%)",
                    100.0 * f64::from(*right) / f64::from(*n)
                );
            }
        }
        s
    }
}

/// Splits examples into what to learn from and what to hold out: every
/// `nth` example is held out, the same ones every time.
pub fn hold_out<T: Clone>(examples: &[T], nth: usize) -> (Vec<T>, Vec<T>) {
    let (mut learn, mut held) = (Vec::new(), Vec::new());
    for (i, e) in examples.iter().enumerate() {
        if nth > 0 && i % nth == nth - 1 {
            held.push(e.clone());
        } else {
            learn.push(e.clone());
        }
    }
    (learn, held)
}
