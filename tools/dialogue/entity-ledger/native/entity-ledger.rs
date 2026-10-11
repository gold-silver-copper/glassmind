//! Offline prospective experiment. Never loaded by MultiModel or draft bundles.
#[path = "../src/entity_ledger_experiment.rs"]
mod experiment;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    experiment::run()
}
