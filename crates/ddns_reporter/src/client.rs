use reqwest::Client;

pub fn init_client() -> Client {
    let client = Client::new();
    client
}
