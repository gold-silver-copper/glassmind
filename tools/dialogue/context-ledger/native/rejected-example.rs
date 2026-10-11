//! Isolated offline contextual source-role experiment, not a runtime adapter.
#[path = "../../../tools/dialogue/context-ledger/native/context_ledger_experiment.rs"]
mod experiment;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    experiment::run()
}
