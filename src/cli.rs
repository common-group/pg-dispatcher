extern crate clap;
use self::clap::{App, Arg};

pub fn create_cli_app<'a, 'b>() -> App<'a, 'b> {
    App::new("pg-dispatcher")
        .version("1.0")
        .about("Listens a PostgreSQL Notification and send through a command execution")
        .arg(Arg::with_name("db-uri")
             .long("db-uri")
             .help("database connection string postgres://user:pass@host:port/dbname")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("channel")
             .long("channel")
             .help("channel to LISTEN")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("exec")
             .long("exec")
             .help("command to execute when receive a notification")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("workers")
             .long("workers")
             .help("max num of workers (threads) to spawn. defaults is 4")
             .required(false)
             .takes_value(true))
}

#[cfg(test)]
mod tests {
    use super::{create_cli_app};

    #[test]
    fn create_cli_app_test() {
        let matches = create_cli_app()
            .get_matches_from(vec![
                              "pg-dispatch", "--db-uri", "foodb",
                              "--channel", "foochan",
                              "--exec", "sh test.sh",
                              "--workers", "5"]);

        assert_eq!("foodb", matches.value_of("db-uri").unwrap());
        assert_eq!("foochan", matches.value_of("channel").unwrap());
        assert_eq!("sh test.sh", matches.value_of("exec").unwrap());
        assert_eq!("5", matches.value_of("workers").unwrap());
    }
}
