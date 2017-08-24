extern crate clap;
use self::clap::{ArgMatches};

#[derive(Debug)]
pub struct DispatcherConfig <'a> {
    pub db_url: &'a str,
    pub db_channel: &'a str,
    pub exec_command: &'a str,
    pub max_threads: usize,
}

impl <'a>DispatcherConfig<'a> {
    pub fn from_matches(matches: &'a ArgMatches<'a>) -> DispatcherConfig {
        let max_threads = match matches.value_of("workers") {
                Some(v) => v.parse::<usize>().unwrap_or(4),
                _ => 4
        };

        DispatcherConfig {
            db_url: matches.value_of("db-uri").unwrap(),
            db_channel: matches.value_of("channel").unwrap(),
            exec_command: matches.value_of("exec").unwrap(),
            max_threads: max_threads
        }
    }
}

#[derive(Debug)]
pub struct Dispatcher<'a> {
    pub config: &'a DispatcherConfig<'a>
}

impl <'a>Dispatcher<'a> {
    pub fn from_config(config: &'a DispatcherConfig<'a>) -> Dispatcher {
        Dispatcher {
            config: config
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use cli;

    #[test]
    fn dispatcher_config_from_matches_test() {
        let matches = cli::create_cli_app()
            .get_matches_from(vec![
                              "pg-dispatch", "--db-uri", "foodb",
                              "--channel", "foochan",
                              "--exec", "sh test.sh",
                              "--workers", "5"]);
        let config = DispatcherConfig::from_matches(&matches);

        assert_eq!(config.db_url, "foodb");
        assert_eq!(config.db_channel, "foochan");
        assert_eq!(config.exec_command, "sh test.sh");
        assert_eq!(config.max_threads, 5);
    }

    #[test]
    fn dispatcher_from_config() {
        let matches = cli::create_cli_app()
            .get_matches_from(vec![
                              "pg-dispatch", "--db-uri", "foodb",
                              "--channel", "foochan",
                              "--exec", "sh test.sh",
                              "--workers", "5"]);
        let config = DispatcherConfig::from_matches(&matches);

        let _disptacher = Dispatcher::from_config(&config);
    }
}
