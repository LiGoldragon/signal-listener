use schema_rust::build::ContractCrateBuild;

fn main() {
    ContractCrateBuild::from_environment(
        "signal-listener",
        "0.2.0",
        "SIGNAL_LISTENER_UPDATE_SCHEMA_ARTIFACTS",
    )
    .expect_fresh();
}
