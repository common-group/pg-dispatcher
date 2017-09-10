extern crate postgres;
extern crate fallible_iterator;

mod cli;
mod dispatcher;
mod thread_pool;

use postgres::{Connection, TlsMode};
use dispatcher::{Dispatcher, DispatcherConfig};
use fallible_iterator::FallibleIterator;
use cli::create_cli_app;
use std::process::{Command, exit};
use std::ffi::OsString;
use std::sync::Arc;


fn main() {
    // parse arguments and build dispatcher
    let cli_matches = create_cli_app().get_matches();
    let config = DispatcherConfig::from_matches(&cli_matches);

    // connect to the database
    let conn = match Connection::connect(config.db_url, TlsMode::None) {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("Failed to connect to the database: {}", error);
            exit(1);
        }
    };
    let _listen_execute = conn.execute(&format!("LISTEN {}", config.db_channel), &[]);
    let notifications = conn.notifications();
    let mut iter = notifications.blocking_iter();

    // instantiate dispatcher
    let dispatcher = Dispatcher::from_config(&config);

    // use a command vector reference
    let command_vector: Arc<Vec<OsString>> = Arc::new(
        config
            .exec_command
            .split_whitespace()
            .map(|s| OsString::from(s))
            .collect(),
    );


    loop {
        match iter.next() {
            Ok(Some(notification)) => {

                let command_vector = Arc::clone(&command_vector);

                dispatcher.pool.execute(move || {
                    let output =
                        Command::new(&command_vector[0])
                            .args(&command_vector[1..])
                            .env("PG_DISPATCH_PAYLOAD", notification.payload)
                            .output()
                            .unwrap_or_else(|e| panic!("failed to execute process: {}\n", e));

                    if output.status.success() {
                        let s = String::from_utf8_lossy(&output.stdout);

                        print!("sh stdout was: {}", s);
                    } else {
                        let s = String::from_utf8_lossy(&output.stderr);

                        print!("rustc failed and stderr was:\n{}\n", s);
                    }
                });
            }
            _ => {}
        }
    }
}
