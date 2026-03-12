use autoschematic_core::tarpc_bridge::tarpc_connector_main;
use connector::DummyConnector;

pub mod connector;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    // Autoschematic loads the connector through the tarpc bridge.
    // After renaming `DummyConnector`, update the type here and leave the
    // rest of the entrypoint alone.
    tarpc_connector_main::<DummyConnector>().await?;
    Ok(())
}
