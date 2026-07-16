use common::utils::TOML;

pub mod gui;
pub mod network;
mod request;

pub const NAME: &str = "Yumush";

const CLIENT_CONFIG_FILE_CONTENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../configs/client_config.toml"
));

pub struct ClientConfig {
    server_address: String,
    server_name: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        let toml = TOML::naive_toml_parser_from_content(CLIENT_CONFIG_FILE_CONTENT).unwrap();

        let server_address = toml.fields.get("server_address").unwrap().to_owned();
        let server_name = toml.fields.get("server_name").unwrap().to_owned();

        Self {
            server_address,
            server_name,
        }
    }
}
