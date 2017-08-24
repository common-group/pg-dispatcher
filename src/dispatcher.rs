#[derive(Debug, Clone)]
pub struct DispatcherConfig <'a> {
    pub db_url: &'a str,
    pub db_channel: &'a str,
    pub exec_command: &'a str,
    pub max_threads: usize,
}

