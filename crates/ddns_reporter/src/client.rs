use std::sync::OnceLock;

use reqwest::Client;
use thiserror::Error;

static CLIENT: OnceLock<Client> = OnceLock::new();

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client initialization error:\n{0}")]
    ClientInitializationError(String),
}

pub fn init_client() -> Result<(), Error> {
    let client = Client::new();
    CLIENT
        .set(client)
        .map_err(|_| Error::ClientInitializationError("Failed to set CLIENT".to_string()))?;
    Ok(())
}

pub fn get_client() -> &'static Client {
    CLIENT
        .get()
        .expect("CLIENT is not initialized. Please call init_client first.")
}
