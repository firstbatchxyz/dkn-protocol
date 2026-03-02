# dkn-protocol

Shared protocol types and wire format for the DKN (Decentralized Knowledge Network) router-node communication layer.

## Overview

This crate defines the message types, framing, authentication handshake, and inference proof structures exchanged between DKN routers and compute nodes over QUIC.

## Modules

| Module | Description |
|---|---|
| `message` | Core message enums (`NodeMessage`, `RouterMessage`) and supporting types (`TaskStats`, `Capacity`, `RejectReason`, etc.) |
| `framing` | Length-prefixed MessagePack read/write over QUIC streams (`read_framed`, `write_framed`) |
| `auth` | Authentication handshake types (`ChallengeMessage`, `AuthRequest`, `AuthResponse`) |
| `proof` | Inference proof structures (`InferenceProof`, `TokenLogprob`) for result validation |
| `template` | Chat template formatting for LLM prompts (ChatML, Llama 3, Gemma) |
| `error` | `ProtocolError` enum for serialization, deserialization, and network errors |

## Wire Format

Messages are serialized with [MessagePack](https://msgpack.org/) and sent as length-prefixed frames:

```
[4-byte BE length][msgpack payload]
```

Maximum message size is 16 MB.

## Usage

```toml
[dependencies]
dkn-protocol = { git = "https://github.com/firstbatchxyz/dkn-protocol.git" }
```

```rust
use dkn_protocol::{RouterMessage, NodeMessage, read_framed, write_framed};
```

## License

Apache-2.0
