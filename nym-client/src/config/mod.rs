use crate::client::SocketType;
use crate::config::template::config_template;
use config::NymConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod persistance;
mod template;

// all of the below are defined in seconds
const DEFAULT_LOOP_COVER_SENDING_AVERAGE_DELAY: f64 = 1.0;
const DEFAULT_MESSAGE_SENDING_AVERAGE_DELAY: f64 = 0.5;
const DEFAULT_AVERAGE_PACKET_DELAY: f64 = 0.2;
const DEFAULT_FETCH_MESSAGES_DELAY: f64 = 1.0;
const DEFAULT_TOPOLOGY_REFRESH_RATE: f64 = 10.0;

#[derive(Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    client: Client,
    socket: Socket,

    #[serde(default)]
    logging: Logging,
    #[serde(default)]
    debug: Debug,
}

impl NymConfig for Config {
    fn template() -> &'static str {
        config_template()
    }

    fn default_root_directory() -> PathBuf {
        dirs::home_dir()
            .expect("Failed to evaluate $HOME value")
            .join(".nym")
            .join("clients")
    }

    fn root_directory(&self) -> PathBuf {
        self.client.nym_root_directory.clone()
    }

    fn config_directory(&self) -> PathBuf {
        self.client
            .nym_root_directory
            .join(&self.client.id)
            .join("config")
    }

    fn data_directory(&self) -> PathBuf {
        self.client
            .nym_root_directory
            .join(&self.client.id)
            .join("data")
    }
}

impl Config {
    pub fn new(id: String) -> Self {
        Config::default().with_id(id)
    }

    pub fn with_id(mut self, id: String) -> Self {
        if self.client.private_identity_key_file.as_os_str().is_empty() {
            self.client.private_identity_key_file =
                self::Client::default_private_identity_key_file(&id);
        }
        if self.client.public_identity_key_file.as_os_str().is_empty() {
            self.client.public_identity_key_file =
                self::Client::default_public_identity_key_file(&id);
        }
        self.client.id = id;
        self
    }

    pub fn with_provider_id(mut self, id: String) -> Self {
        self.client.provider_id = id;
        self
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    /// ID specifies the human readable ID of this particular client.
    id: String,

    /// URL to the directory server.
    directory_server: String,

    /// Path to file containing private identity key.
    private_identity_key_file: PathBuf,

    /// Path to file containing public identity key.
    public_identity_key_file: PathBuf,

    /// provider_id specifies ID of the provider to which the client should send messages.
    /// If initially omitted, a random provider will be chosen from the available topology.
    provider_id: String,

    /// nym_home_directory specifies absolute path to the home nym Clients directory.
    /// It is expected to use default value and hence .toml file should not redefine this field.
    nym_root_directory: PathBuf,
}

impl Default for Client {
    fn default() -> Self {
        // there must be explicit checks for whether id is not empty later
        Client {
            id: "".to_string(),
            directory_server: Self::default_directory_server(),
            private_identity_key_file: Default::default(),
            public_identity_key_file: Default::default(),
            provider_id: "".to_string(),
            nym_root_directory: Config::default_root_directory(),
        }
    }
}

impl Client {
    fn default_directory_server() -> String {
        #[cfg(feature = "qa")]
        return "https://qa-directory.nymtech.net".to_string();
        #[cfg(feature = "local")]
        return "http://localhost:8080".to_string();

        "https://directory.nymtech.net".to_string()
    }

    fn default_private_identity_key_file(id: &str) -> PathBuf {
        Config::default_data_directory()
            .join(id)
            .join("private_identity.pem")
    }

    fn default_public_identity_key_file(id: &str) -> PathBuf {
        Config::default_data_directory()
            .join(id)
            .join("public_identity.pem")
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Socket {
    socket_type: SocketType,
    listening_port: u64,
}

impl Default for Socket {
    fn default() -> Self {
        Socket {
            socket_type: SocketType::None,
            listening_port: 0,
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Logging {}

impl Default for Logging {
    fn default() -> Self {
        Logging {}
    }
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Debug {
    /// The parameter of Poisson distribution determining how long, on average,
    /// sent packet is going to be delayed at any given mix node.
    /// So for a packet going through three mix nodes, on average, it will take three times this value
    /// until the packet reaches its destination.
    /// The provided value is interpreted as seconds.
    average_packet_delay: f64,

    /// The parameter of Poisson distribution determining how long, on average,
    /// it is going to take for another loop cover traffic message to be sent.
    /// If set to a negative value, the loop cover traffic stream will be disabled.
    /// The provided value is interpreted as seconds.
    loop_cover_traffic_average_delay: f64,

    /// The uniform delay every which clients are querying the providers for received packets.
    /// If set to a negative value, client will never try to fetch their messages.
    /// The provided value is interpreted as seconds.
    fetch_message_delay: f64,

    /// The parameter of Poisson distribution determining how long, on average,
    /// it is going to take another 'real traffic stream' message to be sent.
    /// If no real packets are available and cover traffic is enabled,
    /// a loop cover message is sent instead in order to preserve the rate.
    /// If set to a negative value, client will never try to send real traffic data.
    /// The provided value is interpreted as seconds.
    message_sending_average_delay: f64,

    /// Whether loop cover messages should be sent to respect message_sending_rate.
    /// In the case of it being disabled and not having enough real traffic
    /// waiting to be sent the actual sending rate is going be lower than the desired value
    /// thus decreasing the anonymity.
    rate_compliant_cover_messages_disabled: bool,

    /// The uniform delay every which clients are querying the directory server
    /// to try to obtain a compatible network topology to send sphinx packets through.
    /// If set to a negative value, client will never try to refresh its topology,
    /// meaning it will always try to use whatever it obtained on startup.
    /// The provided value is interpreted as seconds.
    topology_refresh_rate: f64,
}

impl Default for Debug {
    fn default() -> Self {
        Debug {
            average_packet_delay: DEFAULT_AVERAGE_PACKET_DELAY,
            loop_cover_traffic_average_delay: DEFAULT_LOOP_COVER_SENDING_AVERAGE_DELAY,
            fetch_message_delay: DEFAULT_FETCH_MESSAGES_DELAY,
            message_sending_average_delay: DEFAULT_MESSAGE_SENDING_AVERAGE_DELAY,
            rate_compliant_cover_messages_disabled: false,
            topology_refresh_rate: DEFAULT_TOPOLOGY_REFRESH_RATE,
        }
    }
}

#[cfg(test)]
mod client_config {
    use super::*;

    #[test]
    fn after_saving_default_config_the_loaded_one_is_identical() {
        let temp_location = std::env::temp_dir().join("config.toml");
        let default_config = Config::default().with_id("foomp".to_string());
        default_config
            .save_to_file(Some(temp_location.clone()))
            .unwrap();

        let loaded_config = Config::load_from_file(Some(temp_location)).unwrap();

        assert_eq!(default_config, loaded_config);
    }
}
