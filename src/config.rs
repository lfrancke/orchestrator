use stackable_config::{ConfigDescription, ConfigOption, Configuration};

pub struct OrchestratorConfig;

impl OrchestratorConfig {
    pub const BIND_ADDRESS: ConfigOption = ConfigOption {
        name: "bind-address",
        default: "0.0.0.0",
        required: false,
        takes_argument: true,
        help: "The address to bind to",
        documentation: "The address to bind to",
    };

    pub const BIND_PORT: ConfigOption = ConfigOption {
        name: "bind-port",
        default: "8080",
        required: false,
        takes_argument: true,
        help: "The port to bind to",
        documentation: "The port to bind to",
    };
}

impl ConfigDescription for OrchestratorConfig {
    fn get_config(&self) -> Configuration {
        Configuration {
            name: "Stackable Orchestrator",
            version: "0.1",
            about: "This is the Orchestrator",
            options: vec![
                OrchestratorConfig::BIND_ADDRESS,
                OrchestratorConfig::BIND_PORT,
            ],
        }
    }
}
