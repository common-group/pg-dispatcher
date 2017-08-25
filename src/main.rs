extern crate postgres;
extern crate fallible_iterator;

mod cli;
mod dispatcher;
mod thread_pool;

use postgres::{Connection, TlsMode};
use dispatcher::{Dispatcher, DispatcherConfig};
use fallible_iterator::FallibleIterator;
use cli::{create_cli_app};

fn main() {
    let cli_matches = create_cli_app().get_matches();
    let config = DispatcherConfig::from_matches(&cli_matches);
    let dispatcher = Dispatcher::from_config(&config);

    let conn = Connection::connect(
        config.db_url, TlsMode::None).unwrap();
    let _listen_execute = conn.execute(
        &format!("LISTEN {}", config.db_channel), &[]);
    let notifications = conn.notifications();
    let mut iter = notifications.blocking_iter();

    loop {
        match iter.next() {
            Ok(Some(notification)) => {
                dispatcher.pool.execute(move||{
                    println!("job should print: {:?}", notification);
                });
            },
            _ => {}
        }
    }
}
