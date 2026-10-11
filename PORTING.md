# Port plan

Reference: `bee-rpc-over-grpc-py` v0.0.2 (`ab3c362`), `src/bee_rpc/`. About 3 500
lines of source and 2 900 lines of tests. Port in this order. Each step is one PR.
Each step ends with tests that use the same vectors as the Python tests.

| Step | Python source | Content |
|---|---|---|
| 1 | `buffer.proto`, `buffer_pb2.py` | Done: `buffer.Buffer` types |
| 2 | `utils.py` (`StreamControl`, `message_to_bytes`, hash helpers), `control.py` | Frame encoding and decoding without gRPC: head, chunk, separator, signal. Unit tests with fixed byte streams |
| 3 | `block_splitter.py`, `block_builder.py`, `block_driver.py` | Blocks: split, hash, nested blocks (up to 32 levels), `previous_lengths_position` |
| 4 | `reader.py`, `validate_lengths_tree.py` | Read and validate a block tree; check the object id |
| 5 | `client.py`: `serialize_to_buffer`, `parse_from_buffer` | Send and receive a stream of messages with partitions. Use `futures` streams and `bytes` |
| 6 | `client.py`: skip signal (`skip_requested_blocks`, `signal_block_buffer_stream`) | The reverse-direction `skip` field and the block store |
| 7 | `client.py`: `client_grpc` | A generic bidi call over tonic: `call(method, [(index, bytes)]) -> [(index, bytes)]` |

Steps 2 to 5 and 7 are what the Celaut light client needs first. Step 6 (skip) and
the disk block store can come later. The phone has no need to hold large blocks on disk.

## Tests

- Take the vectors from `tests/` of the Python repo. `wire_harness.py` and
  `peer_sender.py` show how the tests send bytes between two peers.
- Add an interop test in CI: a Python peer and a Rust peer, in both directions.
  Install `bee-rpc-over-grpc-py` at the tag `v0.0.2`.

## Not in this crate

- The TLS identity check of a Celaut node. It is Celaut-specific and lives in
  `celaut-wire` in chatui.
- The Celaut protobuf types (`celaut.proto`), which are in `celaut-proto` in chatui.
