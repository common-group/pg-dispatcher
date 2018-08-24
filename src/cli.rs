extern crate clap;
use self::clap::{App, Arg};

pub fn create_cli_app<'a, 'b>() -> App<'a, 'b> {
    App::new("pg-dispatcher")
        .version("1.0")
        .about("Listens a PostgreSQL Notification and send through a command execution")
        .arg(Arg::with_name("config")
             .long("config")
             .help("config file path (default: ./pgdispatcher.toml)")
             .required(true)
             .takes_value(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_cli_app_test() {
        let matches = super::create_cli_app()
            .get_matches_from(vec![
                              "pg-dispatch",
                              "--config", "/path/to/pgdispatcher.toml",
            ]);

        assert_eq!("/path/to/pgdispatcher.toml", matches.value_of("config").unwrap());
    }
}
