extern crate clap;

use std::ffi::OsString;
use thread_pool;


#[derive(Debug, Clone)]
pub struct Config<'a> {
    pub db_url: &'a str,
    pub db_channel: &'a str,
    pub max_threads: usize,
    pub command_vector: Vec<OsString>,
}

impl<'a> Config<'a> {
    pub fn from_matches(matches: &'a clap::ArgMatches<'a>) -> Config {
        let max_threads = match matches.value_of("workers") {
            Some(v) => v.parse::<usize>().unwrap_or(4),
            _ => 4,
        };

        let command_vector: Vec<OsString> = matches
            .value_of("exec")
            .unwrap()
            .split_whitespace()
            .map(|s| OsString::from(s))
            .collect();

        Config {
            db_url: matches.value_of("db-uri").unwrap(),
            db_channel: matches.value_of("channel").unwrap(),
            max_threads: max_threads,
            command_vector: command_vector,
        }
    }
}

#[derive(Debug)]
pub struct Dispatcher {
    pub pool: thread_pool::ThreadPool,
}

impl Dispatcher {
    pub fn from_config(config: &Config) -> Dispatcher {
        Dispatcher {
            pool: thread_pool::ThreadPool::new(config.max_threads, config.command_vector.clone()),
        }
    }

    pub fn execute_command(&self, payload: String) {
        self.pool.execute(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cli;

    #[test]
    fn dispatcher_config_from_matches_test() {
        let matches = cli::create_cli_app().get_matches_from(vec![
            "pg-dispatch",
            "--db-uri",
            "foodb",
            "--channel",
            "foochan",
            "--exec",
            "sh test.sh",
            "--workers",
            "5",
        ]);
        let config = Config::from_matches(&matches);

        assert_eq!(config.db_url, "foodb");
        assert_eq!(config.db_channel, "foochan");
        assert_eq!(config.exec_command, "sh test.sh");
        assert_eq!(config.max_threads, 5);
    }

    #[test]
    fn dispatcher_from_config() {
        let matches = cli::create_cli_app().get_matches_from(vec![
            "pg-dispatch",
            "--db-uri",
            "foodb",
            "--channel",
            "foochan",
            "--exec",
            "sh test.sh",
            "--workers",
            "5",
        ]);
        let config = Config::from_matches(&matches);

        let _disptacher = Dispatcher::from_config(&config);
    }
}
