# Prompt for the implementation agent

Give the text below to the agent, as it is. Replace `<N>` with the step to do.

---

You are a Rust engineer. Implement step `<N>` of the Rust port of Bee-RPC in the
repository https://github.com/bee-rpc-protocol/bee-rpc-over-grpc-rs (branch
`master`).

Before you change anything, read these files completely, in this order:
`AGENT_GUIDE.md`, `README.md`, `PORTING.md`, `DEVIATIONS.md`, then the doc comments
of the modules of step `<N>` in `crates/bee-rpc/src/`, and the Python source that
`PORTING.md` names for that step. The Python reference is
https://github.com/bee-rpc-protocol/bee-rpc-over-grpc-py at tag `v0.0.2`.

Your goal: the test files of step `<N>` (table in `PORTING.md`) pass, and all the
checks of `AGENT_GUIDE.md` section 4 pass.

Rules (the full list is in `AGENT_GUIDE.md` section 2):

1. The tests and the vectors in `crates/bee-rpc/tests/` are the specification.
   They come from the Python reference. Do not change, skip or delete a test, a
   vector, or any file in `tools/`, `.github/` (except `required-tests.txt`),
   `DEVIATIONS.md` or `SPEC.sha256`.
2. If you think that a test is wrong, stop. Write a report with the test name, the
   case name, what you expected, and the output of the Python reference on the same
   input. Do not try to make the test pass in another way.
3. No code in `crates/bee-rpc/src/` knows about the tests: no vector names, no
   paths of `tests/`, no special case for one input.
4. No `unsafe`, no `unwrap`/`expect`/`panic!` on data from a peer, no new
   `#[allow]`, no blocking calls in async code.
5. Do not change a public signature in `crates/bee-rpc/src/`. Do not remove a
   public item.
6. Use only the dependencies that `AGENT_GUIDE.md` section 2.5 allows.
7. Do only step `<N>`. Do not start the work in "Not in this plan" of `PORTING.md`.

When you are done:

- Add the test names of step `<N>` to `.github/required-tests.txt`.
- Make a branch `port/step-<N>`, commit with short imperative messages, and open a
  PR to `master`.
- In the PR text, write: what you implemented, the output of the checks, the public
  items that you added (if any) and why, and your open questions.

If you cannot finish, open the PR as a draft and say which tests fail and why.
