use schema_rust::build::ContractCrateBuild;

fn main() {
    ContractCrateBuild::from_environment(
        "signal-listener",
        "0.1.0",
        "SIGNAL_LISTENER_UPDATE_SCHEMA_ARTIFACTS",
    )
    .expect_fresh();
}
