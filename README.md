# lurk-sansio

A sans-IO game engine for the [Lurk protocol](https://github.com/sethlong/lurk) created by S. Seth Long, Ph.D.

All game logic lives here. There are no sockets, no async runtimes, and no threads. You feed `Input` events in, poll `Output` events out, and your event loop handles the actual networking.

## Usage

```rust
use std::collections::HashMap;
use lurk_sansio::{GameEngine, GameConfig, Input, Output, ClientId};

let rooms = HashMap::new();
let config = GameConfig { initial_points: 100, stat_limit: 65535 };
let mut engine = GameEngine::new(rooms, config);

// Feed events from your event loop
engine.handle_input(Input::ClientConnected { client: ClientId(1) });

// Poll outputs and send them over the wire
while let Some(output) = engine.poll_output() {
    match output {
        Output::SendError { client, error_code, message } => { /* send error packet */ }
        Output::Disconnect { client } => { /* close connection */ }
        _ => { /* handle other outputs */ }
    }
}
```

## Key Types

| Type         | Purpose                                                    |
| ------------ | ---------------------------------------------------------- |
| `GameEngine` | Holds all game state and processes inputs                  |
| `Input`      | Events the event loop feeds into the engine                |
| `Output`     | Side-effects the engine produces for the event loop        |
| `Character`  | Player or monster stats and status                         |
| `Room`       | A location in the game world with connections and monsters |
| `GameConfig` | Character creation constraints                             |

## Design

- **Testable** - no mocking needed, just push inputs and assert outputs
- **Portable** - runs anywhere Rust compiles (server, WASM, test harness)
- **Deterministic** - same inputs always produce the same outputs
