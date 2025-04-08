# KagiKachi
**KagiKachi** is a simple key-value storage written in rust


## Current state of the project:
- custom implementations of SHA1 and base64
- working websocket connection
- basic cache database allowing for storing and retrieving data, with option to traverse structures

## Finished, but require testing
- Websockets:
  - wrapper for websocket server 
  - handshake and ping automation
  - generic inner data storage
  - handler function should accept request payload and inner data, and return response payload (or possibly subscription to some event)
- Cache database:
  - basic command parser (commands should be 3 of 4 letters long)
  - basic command handler

## Todo:
- Cache database:
  - more commands
  - tests

## Building
```sh
cargo build --release
```
Compiled binary file is located inside /target/release/

## Testing
```sh
cargo test
```

App starts websocket server on port 7878