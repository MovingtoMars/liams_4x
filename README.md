# Liam's 4x Game

## Demo

[Demo video](https://streamable.com/0bmjhb)

## Building

The two big dependencies are ggez and imgui.

ggez build requirements:
https://github.com/ggez/ggez/blob/master/docs/BuildingForEveryPlatform.md

imgui build requirements:
https://github.com/Gekkio/imgui-rs

rust-clipboard build requirements:
https://github.com/aweinstock314/rust-clipboard

Basically, on Windows you should be using the MSVC ABI version of Rust.

After all the requirements are ready:
```sh
cargo run
```

## Architecture

The code is split into three top-level modules:

* common, for code used by client and server (including all game logic)
* client
* server

There are two elements which are used to update game state:

* actions, which represent a user interaction with the world
* events, which apply a state change the world

This is the sequence for updating game state:
1. The user does something in the client that interacts with the game world.
1. An action is generated and sent to the server. The action doesn't directly update any game state.
1. The server looks at the world state and the incoming action, then emits some number of events
  (possibly 0 events if there is no change or the action is invalid).
1. The server applies the events to its copy of the game state.
1. The server sends the events to each client.
1. Each client applies the events to its copy of the game world.
