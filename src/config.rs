extern crate clap;
extern crate toml;

//#[macro_use]
//extern crate serde_derive;

use std::fs::File;
//use std::ffi::OsString;
use std::io::prelude::*;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub channels: Vec<ChannelConfig>
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelConfig{
    pub name: String,
    pub producer: Option<ChannelProducerConfig>,
    pub consumer: Option<ChannelConsumerConfig>,
    pub redis: ChannelRedisConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelProducerConfig {
    pub listen_channel: String,
    pub postgres_uri: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelConsumerConfig {
    pub workers: usize,
    pub command: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChannelRedisConfig {
    pub uri: String
}

impl Config {
    pub fn from_matches(matches: &clap::ArgMatches) -> Config {
        let mut config_file = File::open(matches
            .value_of("config")
            .unwrap_or("pgdispatcher.toml")
            ).expect("config file not found");

        let mut config_contents = String::new();
        config_file.read_to_string(&mut config_contents)
            .expect("file to reading config file");
        debug!("READED CONFIG ---> {}", config_contents);

        let decoded_config: Config = toml::from_str(config_contents.as_ref())
            .expect("invalid config");

        debug!("[DEBUG] DECODED CONFIG ---> {:?}", decoded_config);

        return decoded_config;
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use cli;

    #[test]
    fn config_from_valid_file() {
        let matches = cli::create_cli_app()
            .get_matches_from(vec![
                              "pg-dispatch",
                              "--config", "tests/config/example.toml",
        ]);
        let config = Config::from_matches(&matches);
        let both_channel = &config.channels[0];
        let consumer_channel = &config.channels[1];

        assert_eq!(both_channel.name, "payment_stream_producer");
        assert_eq!(both_channel.consumer.is_some(), true);
        assert_eq!(both_channel.producer.is_some(), true);
        assert_eq!(both_channel.redis.uri, "redis://rediskey@localhost:6973/0");

        assert_eq!(consumer_channel.name, "payment_stream_consumer");
        assert_eq!(consumer_channel.consumer.is_some(), true);
        assert_eq!(consumer_channel.producer.is_some(), false);
        assert_eq!(consumer_channel.redis.uri, "redis://rediskey@localhost:6973/0");
    }
}
