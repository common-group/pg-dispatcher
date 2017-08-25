### under development

# pg-dispatcher
Abstract listener for PostgreSQL, that listen ah single channel and dispatch the messages via any command execution of database.

## install

`git clone git@github.com:common-group/pg-dispatcher.git`

`cd pg-dispatcher`

`cargo build --release`

## usage (example)

create a file to process the payload message:

`echo 'echo $PG_DISPTACH_PAYLOAD' > test.sh`

running compiled bin:
```
./target/release/pg-disptacher --db-uri='postgres://postgres@localhost/postgres' \ 
  --channel="test_channel" \
  --exec="sh ./test.sh" \
  --workers=100
```

connect at you database and execute:
```
  select pg_notify('test_channel', json_build_object('id', '1')::text);
```


