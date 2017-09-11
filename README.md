### under development

# pg-dispatch

Abstract listener for PostgreSQL, that listens to a single database channel and executes a
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
    pg-dispatcher [OPTIONS] --db-uri <db-uri> --channel <channel> --exec <exec>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --channel <channel>    channel to LISTEN
        --db-uri <db-uri>      database connection string postgres://user:pass@host:port/dbname
        --exec <exec>          command to execute when receive a notification
        --workers <workers>    max num of workers (threads) to spawn. defaults is 4
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
      --channel="test_channel"                           \
      --exec=cat                                         \
      --workers=100
```

Then, connect to your PostgreSQL database and execute the following command to issue a
notification through the `tests_channel` channel:

```sql
postgres=# NOTIFY test_channel, 'hello from postgres';
```

The console will then have the following output:

```
[pg-dispatch] Listening to channel: "test_channel".
[worker-1] Got payload: hello from postgres.
[worker-1] Command succeded with status code 0.
[cat-1] hello from postgres
```

#### Dispatching a command with arguments

You can also use commands with arguments, just pass them inside the same string:.

```sh
$ ./target/release/pg-disptacher                         \
      --db-uri='postgres://postgres@localhost/postgres'  \
      --channel="test_channel"							 \
	  --exec="sh some-script.sh"                         \
      --workers=100
```

Where `some-script.sh` could be like:

```sh
#!/bin/sh

PAYLOAD=$(cat) # read from stdin
echo "The payload was: $PAYLOAD!"
```

Output *(after notification was issued)*:
```
[worker-1] Got payload: hello from postgres.
[worker-1] Command succeded with status code 0.
[sh-1] The payload was: hello from postgres!
```
