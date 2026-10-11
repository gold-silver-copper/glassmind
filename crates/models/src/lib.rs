//! Small models glassmind learns from data. They propose and rank; the
//! typed code decides. Each is learned by counting, can be written out as
//! plain text to read (its dump is the model), and knows nothing of any
//! language or world: a language says what its tokens are and what to do
//! with what a model proposes.

pub mod edits;
pub mod intents;
pub mod report;

pub use edits::{Edit, EditModel};
pub use intents::IntentModel;
pub use report::Calibration;
