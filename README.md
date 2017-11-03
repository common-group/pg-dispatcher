### under development

### TODO
- [ ] Can have multiple dispatcher running on same channel with diff commands
- [ ] Add --retry option and logic to remove failed executions from (done, processing) and count retries
- [ ] Maybe add --retry-on-code=[1, ...] retry on custom exit codes
- [ ] Build with alpine docker (waiting for rust 1.18 on packages)


# pg-dispatch

Abstract listener for PostgreSQL that listens to a single database channel and executes a
given command when a notification arrives. The notification payload, if any, is sent
though the executed command's standard input.

## Installation

1. `$ git clone git@github.com:common-group/pg-dispatcher.git`
2. `$ cd pg-dispatcher`
3. `$ cargo build --release`

## Usage

```
$ pg-dispatcher --help

pg-dispatcher 1.0
Listens a PostgreSQL Notification and send through a command execution

USAGE:
    pg-dispatcher [OPTIONS] --db-uri <connection-string> --redis-uri <redis-uri> --channel <channel> --exec <exec>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --channel <channel>        channel to LISTEN
        --db-uri <db-uri>          database connection string postgres://user:pass@host:port/dbname
        --exec <exec>              command to execute when receive a notification
        --mode <mode>              consumer, producer or both (default both)
        --redis-uri <redis-uri>    redis connection string redis://localhost:6379
        --workers <workers>        max num of workers (threads) to spawn. defaults is 4
```

### Examples

#### Dispatching a command without arguments

The example below will listen to a PostgreSQL database channel named `test_channel` and
execute the command `cat` whenever a new notification arrives, using `100` threads at
maximum.

*(Note that the `cat` command reads from standard input when no file is specified)*

```sh
$ ./target/release/pg-disptacher                         \
      --db-uri='postgres://postgres@localhost/postgres'  \
      --redis-uri='redis://localhost:6379'               \
      --channel="test_channel"                           \
      --exec=cat                                         \
      --workers=10
```

Then, connect to your PostgreSQL database and execute the following command to issue a
notification through the `tests_channel` channel:

```sql
postgres=# NOTIFY test_channel, 'hello from postgres';
```

The console will then have the following output:

```
[pg-dispatcher-producer] Producer Listening to channel: "test_channel".
[pg-dispatcher-consumer] Start consumer for payloads of channel test_channel
[pg-dispatcher-producer] received key aGVsbG8gZnJvbSBwb3N0Z3Jlcw==
[pg-dispatcher-consumer] start processing key aGVsbG8gZnJvbSBwb3N0Z3Jlcw==
[worker-0] Got payload: hello from postgres.
[worker-0] Command succeded with status code 0.
[cat-0] hello from postgres
```

#### Dispatching a command with arguments

You can also use commands with arguments, just pass them inside the same string:.

```sh
$ ./target/release/pg-disptacher                         \
      --db-uri='postgres://postgres@localhost/postgres'  \
      --redis-uri='redis://localhost:6379'               \      
      --channel="test_channel"                           \
      --exec="sh some-script.sh"                         \
      --workers=100
```

Where `some-script.sh` could be like:

```sh
#!/bin/sh
PAYLOAD=$(cat) # read from stdin
echo "The payload was: $PAYLOAD!"
```
