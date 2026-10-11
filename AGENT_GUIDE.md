# Guide for the agent that implements the port

Read this file completely before you change anything. Read `README.md`,
`PORTING.md` and `DEVIATIONS.md` after it.

## 1. The task

Implement the Rust crate `bee-rpc` (`crates/bee-rpc/src/`) so that the tests in
`crates/bee-rpc/tests/` pass. The tests and their vectors are the specification.
The vectors come from the Python reference (bee-rpc-over-grpc-py v0.0.2), not from
a person. Do steps 2 to 7 of `PORTING.md`, in order, one PR per step.

## 2. Safeguards

These rules are not optional. A PR that breaks one is not merged.

### 2.1 Files that you must not change

- `crates/bee-rpc/tests/**` (tests, helpers and vectors)
- `tools/vectors/**`, `tools/interop/**`
- `DEVIATIONS.md`, `SPEC.sha256`, `tools/spec-hash.sh`, `AGENT_GUIDE.md`, `PROMPT.md`
- `crates/bee-rpc/protos/**`, `crates/bee-rpc/src/gen/**`
- `.github/**`, except that you add lines to `.github/required-tests.txt`

CI checks the first group with `tools/spec-hash.sh --check`. If you change one of
these files, CI fails.

### 2.2 If a test looks wrong

Do not change the test, do not skip it, and do not make the code detect it. Stop
the step, and write a report:

1. The test name and the vector case name.
2. What the test expects and what you think is correct.
3. Evidence from the reference: run the Python library (section 5) on the same
   input and show the output.

Then wait for a maintainer. The maintainer changes the generator or the test.

### 2.3 Forbidden in `crates/bee-rpc/src/`

- Code that knows about the tests: no vector names, no `manifest.json`, no file
  paths of `tests/`, no special case for one input. CI greps for this.
- `unsafe` (the crate has `#![forbid(unsafe_code)]`).
- `unwrap`, `expect`, `panic!`, and indexing that can panic, on any path that reads
  data from a peer. The crate denies `clippy::unwrap_used`, `clippy::expect_used`
  and `clippy::panic`. Do not add `#[allow]` for these or for any other lint.
- `todo!()` or `unimplemented!()` in a module of a finished step.
- Blocking calls (file I/O, `std::thread::sleep`, blocking locks held across an
  `.await`) in async code.
- File or network I/O in `hash`, `pointer`, `frame`, `skip`, `signal`. They work on
  memory only. `bee_file` uses only the `Read` and `Write` that it gets.

### 2.4 The public API

The public items in `src/` are the contract. Do not remove a public item, and do
not change a public signature. You can add private items. If you must add a public
item, say so in the PR and explain why.

### 2.5 Dependencies

You can add these crates: `sha2`, `sha3`, `blake2`, `tonic`, `tonic-prost`,
`tokio`, `tokio-stream`, `futures`, `bytes`, `http`, `http-body`, `tower`. Use a
version that is at least 2 weeks old. Put gRPC crates behind the `grpc` feature.
Ask before you add any other crate.

### 2.6 Memory

- `serialize_part` makes each chunk when the iterator asks for it. It does not copy
  the whole part more than once.
- The parser keeps one copy of the open message and one hasher per open block. It
  does not keep the Buffers it already used.

## 3. How to do one step

1. Make a branch `port/step-N` from `master`.
2. Read the module docs in `src/` and the Python source in the table of
   `PORTING.md`.
3. Implement. Run the tests of the step:
   `cargo test --all-features --test <name>`.
4. Add the test names of the step to `.github/required-tests.txt`. Do not remove a
   line.
5. Run all the checks in section 4. All must pass.
6. Make one PR per step. The PR text has: what you implemented, the test results,
   new public items (if any), and the questions that you have.

Do not start the next step before the PR of the step is merged, unless a maintainer
tells you to.

## 4. Checks (all must pass before a PR)

```sh
tools/spec-hash.sh --check
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --all-features --no-run
# the tests of every finished step:
while read -r t; do case "$t" in ''|'#'*) continue;; esac
  cargo test --all-features --test "$t" || break; done < .github/required-tests.txt
```

Step 7 needs the Python peer: set `BEE_RPC_PYTHON` (section 5). In CI a missing
`BEE_RPC_PYTHON` makes the interop tests fail, not skip.

## 5. The Python reference

Make a Python 3.11 environment with the pinned versions:

```sh
python3.11 -m venv .venv-ref
.venv-ref/bin/pip install -r tools/vectors/requirements.txt
export BEE_RPC_PYTHON="$PWD/.venv-ref/bin/python"
```

Use it to see what the reference does with an input. Run Python with `-I`. Make the
library use temporary directories first, because it writes `__cache__` and
`__block__` in the current directory:

```python
import tempfile, os
from bee_rpc.utils import modify_env
d = tempfile.mkdtemp(); os.makedirs(d + '/b')
modify_env(cache_dir=d + '/', block_dir=d + '/b/')
```

`tools/vectors/generate.py` shows how each vector was made. Do not commit
`.venv-ref`.

## 6. Facts about the reference that are easy to miss

- `Head { index: 0 }` is present on the wire for raw bytes. It encodes as an empty
  message, but the field is set.
- `chunk` is set (to empty bytes) in the one-Buffer form of an empty message.
- A message of exactly `CHUNK_SIZE` bytes uses the long form: head, one chunk,
  separator.
- A block id is the hash of the block expansion, nested blocks included. The
  parser must feed the bytes of a nested block to the hashers of all the blocks
  around it. When a block comes from the store, feed the store bytes to them too.
- The end marker of a block is the same pointer as its start marker. The block id
  says which marker it is: the innermost open id closes, an id that is not open
  opens.
- Message mode of the reference is not the expectation for streams with blocks
  (`DEVIATIONS.md`, D1).
- The skip filter checks the skip set again for each Buffer. A request can arrive in
  the middle of a block.
