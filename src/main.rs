extern crate postgres;
extern crate fallible_iterator;

mod cli;
mod dispatcher;
mod thread_pool;

use cli::create_cli_app;
use dispatcher::{Dispatcher, Config};
use fallible_iterator::FallibleIterator;
use postgres::{Connection, TlsMode};
use std::process::exit;


fn main() {
    // consume cli arguments
    let cli_matches = create_cli_app().get_matches();
    let config = Config::from_matches(&cli_matches);

    // connect to database
    let conn = match Connection::connect(config.db_url, TlsMode::None) {
        Ok(conn) => conn,
        Err(error) => {
            eprintln!("Failed to connect to the database: {}.", error);
            exit(1);
        }
    };
    if let Err(_) = conn.execute(&format!("LISTEN {}", config.db_channel), &[]) {
        eprintln!("Failed to execute LISTEN command in database.");
        exit(1)
    }

    println!(
        "[pg-dispatch] Listening to channel: \"{}\".",
        config.db_channel
    );

    // make an iterator over notifications
    let notifications = conn.notifications();
    let mut iter = notifications.blocking_iter();

    let dispatcher = Dispatcher::from_config(&config);

    loop {
        match iter.next() {
            Ok(Some(notification)) => dispatcher.execute_command(notification.payload),
            _ => {}
        }
    }
}
