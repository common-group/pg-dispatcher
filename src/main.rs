extern crate postgres;
extern crate fallible_iterator;

mod cli;
mod dispatcher;
mod thread_pool;

use postgres::{Connection, TlsMode};
use dispatcher::{Dispatcher, DispatcherConfig};
use fallible_iterator::FallibleIterator;
use cli::create_cli_app;
use std::process::Command;

fn main() {
    let cli_matches = create_cli_app().get_matches();
    let config = DispatcherConfig::from_matches(&cli_matches);
    let dispatcher = Dispatcher::from_config(&config);

    let conn = Connection::connect(config.db_url, TlsMode::None).unwrap();
    let _listen_execute = conn.execute(&format!("LISTEN {}", config.db_channel), &[]);
    let notifications = conn.notifications();
    let mut iter = notifications.blocking_iter();

    loop {
        match iter.next() {
            Ok(Some(notification)) => {
                let cmd_handler = config.exec_command.to_string();

                dispatcher.pool.execute(move || {
                    let split = cmd_handler.split_whitespace();
                    let cmd_vector = split.collect::<Vec<&str>>();
                    let output =
                        Command::new(cmd_vector[0])
                            .args(&cmd_vector[1..cmd_vector.len()])
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
