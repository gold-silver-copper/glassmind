//! Reusable, non-generative intent proposals and explicit conversation state.
//! Labels and features are learned exclusively from the supplied training set.
//! Validation chooses a refusal policy; it never contributes model weights.

pub mod adapter;
pub mod memory;
pub mod model;

pub use adapter::{Example, Totals};
pub use memory::{Conversation, Reply};
pub use model::{IntentModel, Prediction, TrainOptions};
