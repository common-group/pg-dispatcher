mod cli;
mod dispatcher;

use dispatcher::{listener, DispatcherConfig};
use cli::{create_cli_app};

fn main() {
    listener(
        &DispatcherConfig::from_matches(
            &create_cli_app().get_matches()));
}
