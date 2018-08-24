extern crate clap;
extern crate postgres;
extern crate redis;
extern crate fallible_iterator;
extern crate base64;

use config::{Config, DispatcherConfig};
use std::ffi::OsString;
use self::fallible_iterator::FallibleIterator;
use thread_pool::{ThreadPool, WorkerMessage};
use std::str;
use std::process::exit;
use std::{thread, time};
use redis::Commands;
//pub command_vector: Vec<OsString>,
//command_vector: matches.value_of("exec")
//    .unwrap()
//    .split_whitespace()
//    .map(|s| OsString::from(s))
//    .collect(),

#[derive(Debug)]
pub struct DispatcherPool<'a> {
    pub config: &'a Config,
    pub pool: Vec<Dispatcher>
}

#[derive(Debug)]
pub struct Dispatcher {
    pub config: DispatcherConfig,
}

impl <'a>DispatcherPool<'a> {
    pub fn from_config(config: &'a Config) -> DispatcherPool<'a> {
        let pool = DispatcherPool {
            config: config,
            pool: config
                .dispatchers
                .iter()
                .map(|d| Dispatcher::from_config(&d))
                .collect()
        };
        debug!("DispatcherPool from config {:?}", pool);
        return pool;
    }
}

impl Dispatcher {
    pub fn from_config(config: &DispatcherConfig) -> Dispatcher {
        let dispatcher = Dispatcher {
            config: config.clone()
        };
        debug!("Dispatcher from config {:?}", dispatcher);
        return dispatcher;
    }

    pub fn start_consumer(&self) -> Option<thread::JoinHandle<()>> {
        {
            if(!config.consumer.is_some()) {
                debug!("not have a consumer configuration. customer not started");
                return None;
            };

            let redis_client = redis::Client::open(config.redis.uri.as_str()).unwrap();
            let redis_conn = redis_client.get_connection().unwrap();
            let pending_set = format!("dispatcher:{}:pending_set", &config.db_channel);
            let processing_set = format!("dispatcher:{}:processing_set", &config.db_channel);
            let done_set = format!("dispatcher:{}:done_set", &config.db_channel);

            let handler = thread::spawn(move||{
                let pool = ThreadPool::new(
                    config.max_threads,
                    config.command_vector.clone());

                println!(
                    "[pg-dispatcher-consumer] Start consumer for payloads of channel {}",
                    config.db_channel);

                loop {
                    let guard_idle_counter = pool.idle_counter.clone();
                    let guard_counter: usize;

                    {
                        let counter = match guard_idle_counter.lock() {
                            Ok(count) => count,
                            Err(p) => p.into_inner()
                        };

                        guard_counter = *counter;
                    }

                    if let Ok(worker_output) = pool.workers_channel.try_recv() {
                        match worker_output {
                            WorkerMessage::ProgramNotFound(b64_key)
                                | WorkerMessage::StdinFailed(b64_key) => {
                                let _ : Result<(),_> = redis_conn.
                                    srem(processing_set.clone(), b64_key);
                            },
                            WorkerMessage::DoneTask(b64_key) => {
                                // add to done task
                                // TODO: add some task to cleanup the done set
                                let _ : Result<(), _> = redis_conn.
                                    sadd(done_set.clone(), b64_key.clone());

                                // remove from pending set
                                let _ : Result<(),_> = redis_conn.
                                    srem(pending_set.clone(), b64_key.clone());

                                // remove from processing set
                                let _ : Result<(),_> = redis_conn.
                                    srem(processing_set.clone(), b64_key.clone());
                            }
                        }
                    }

                    let diff_result : Result<Vec<String>, _> = redis_conn
                        .sdiff(&[pending_set.clone(), processing_set.clone()]);

                    if let Ok(diff) = diff_result {
                        for (i, key) in diff.iter().enumerate() {
                            if i+1 > guard_counter { break; }
                            let decoded = base64::decode(&key).unwrap();

                            if let Ok(payload) = str::from_utf8(&decoded) {
                                match redis_conn.sadd(processing_set.clone(), key) {
                                    Ok(1) => {
                                        println!("[pg-dispatcher-consumer] start processing key {}", &key);
                                        pool.execute(payload.to_string())
                                    },
                                    _ => {}
                                };
                            }
                        }
                    }

                    thread::sleep(time::Duration::from_millis(100));
                }
            });

            return handler;
        }
    }

//    pub fn start_producer(&self, pg_conn: postgres::Connection, redis_client: redis::Client) -> thread::JoinHandle<()> {
//        {
//            let config = self.config.clone();
//            let pending_set = format!("dispatcher:{}:pending_set", &config.db_channel);
//            if let Err(_) = pg_conn.execute(&format!("LISTEN {}", config.db_channel), &[]) {
//                eprintln!("Failed to execute LISTEN command in database.");
//                exit(1)
//            }
//
//            let handler = thread::spawn(move||{
//                println!(
//                    "[pg-dispatcher-producer] Producer Listening to channel: \"{}\".",
//                    config.db_channel
//                    );
//
//                let notifications = pg_conn.notifications();
//                let mut iter = notifications.blocking_iter();
//
//                loop {
//                    match iter.next() {
//
//                        Ok(Some(notification)) => {
//                            let key_value = base64::encode(&notification.payload);
//                            println!("[pg-dispatcher-producer] found new notification {:?}", &key_value);
//                            let redis_conn = redis_client.get_connection().unwrap();
//                            match redis_conn.sadd(pending_set.clone(), &key_value) {
//                                Ok(1) => {
//                                    println!("[pg-dispatcher-producer] received key {}", &key_value);
//                                },
//                                Err(error) => {
//                                    println!("[pg-dispatcher-producer] error {:?}", error);
//                                },
//                                _ => {
//                                    println!("[pg-dispatcher-producer] key {} already persisted", &key_value);
//                                }
//                            };
//                        },
//                        _ => {}
//                    }
//                }
//            });
//
//            return handler;
//        }
//    }
//
}

#[cfg(test)]
mod tests {
    use super::*;
    use cli;

    #[test]
    fn dispatcher_pool_from_config() {
        let matches = cli::create_cli_app()
            .get_matches_from(vec![
                              "pg-dispatch",
                              "--config", "tests/config/example.toml",
        ]);
        let config = Config::from_matches(&matches);
        let dispatcher_pool = DispatcherPool::from_config(&config);
        let dispatchers = dispatcher_pool.pool;
        let dispatcher_both = &dispatchers[0];
        let dispatcher_consumer = &dispatchers[1];

        assert_eq!(dispatcher_both.config.consumer.is_some(), true);
        assert_eq!(dispatcher_both.config.producer.is_some(), true);
        assert_eq!(dispatcher_both.config.redis.uri, "redis://rediskey@localhost:6973/0");

        assert_eq!(dispatcher_consumer.config.consumer.is_some(), true);
        assert_eq!(dispatcher_consumer.config.producer.is_some(), false);
        assert_eq!(dispatcher_consumer.config.redis.uri, "redis://rediskey@localhost:6973/0");
        //assert_eq!(
        //    config.command_vector,
        //    vec![OsString::from("sh"), OsString::from("test.sh")]
        //    );
    }
}
