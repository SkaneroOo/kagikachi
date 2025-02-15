# KagiKachi
**KagiKachi** is a simple key-value storage written in rust

Name comes from japanese 鍵 (kagi) meaning key, and 価値 (kachi) meaning value

## Current state of the project:
- custom implementations of SHA1 and base64 (still have to be optimized, especially base64 decoding)
- working websocket connection
- basic interfaces simplifying recieving and sending packets

## Finished, but require testing
- Websockets:
  - wrapper for websocket server 
  - handshake and ping automation
  - generic inner data storage
  - handler function should accept request payload and inner data, and return response payload (or possibly subscription to some event)

## Todo:
- Cache database:
  - basic command parser (commands should be 3 of 4 letters long)
  - basic command handler