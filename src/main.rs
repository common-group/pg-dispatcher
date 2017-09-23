extern crate postgres;
extern crate redis;

mod cli;
mod dispatcher;
mod thread_pool;

use cli::create_cli_app;
use dispatcher::{Dispatcher, Config};
use std::process::exit;
use std::thread;
use postgres::{TlsMode};
use postgres::tls::native_tls::NativeTls;

fn main() {
    let cli_matches = create_cli_app().get_matches();
    let config = Config::from_matches(&cli_matches);
    let dispatcher = Dispatcher::from_config(&config);
    let redis_client = redis::Client::open(config.redis_url.as_str()).unwrap();
    let mut _servers: Vec<thread::JoinHandle<()>> = Vec::new();

    if config.producer {
        let negotiator = NativeTls::new().unwrap();
        let tls_mode : TlsMode = match config.tls_mode.as_ref() {
            "prefer" => { TlsMode::Prefer(&negotiator) },
            "require" => { TlsMode::Require(&negotiator) },
            _ => { TlsMode::None },
        };
        let pg_conn = match postgres::Connection::connect(
            config.db_url.as_str(), tls_mode
         ) {
            Ok(conn) => conn,
            Err(error) => {
                eprintln!("Failed to connect to the database: {}.", error);
                exit(1);
            }
        };

        _servers.push(
            dispatcher.start_producer(
                pg_conn, redis_client.clone()));
    }

    if config.consumer {
        _servers.push(
            dispatcher.start_consumer(
                redis_client.clone()));
    }

    for _server in _servers {
        let _ = _server.join();
    }
}
