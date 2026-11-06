# Libertas Tax

U.S. federal tax year 2026 return prototype: a guided interview, source-based
calculations, and on-screen estimates. Printable forms and electronic filing
are deferred. See [coverage and verification](COVERAGE.md) for the supported
scope and its limits, and [code metrics](CODE-METRICS.md) for source size.

The repository is [LibertasIoT/libertas-tax](https://github.com/LibertasIoT/libertas-tax).
It holds the App source, its display template (`display.json`) and its tests.
The [federal tax example](https://github.com/LibertasIoT/libertas-doc/blob/main/examples/federal-tax.md)
in [libertas-doc](https://github.com/LibertasIoT/libertas-doc) describes what the
App does and how its interview works.

For development, start with [CODEBOT.md](CODEBOT.md). The platform's
[App authoring contract](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/app-contract.md)
and [Rust language reference](https://github.com/LibertasIoT/libertas-doc/blob/main/reference/languages/rust.md)
are published in libertas-doc.

## Build and run

`Cargo.toml` pins the runtime and macro crates of the public
[libertas-rs](https://github.com/LibertasIoT/libertas-rs) SDK to an exact Git
revision, so no sibling checkout is needed. With a current stable Rust
toolchain:

```sh
cargo build --locked
cargo test --locked
```

Display rules are authored in `display/<form>.json`; `display.json` is assembled
by `tools/assemble-display`. Each source entry has an explicit integer `order`
and a normal Display Template `rule`. Keep existing order values stable; use
an unused value between neighboring rules when inserting one. Assembly sorts
by this order across all fragments, preserving override precedence. Run the
assembler after editing a fragment and regenerate the output when merging
fragment changes.

Before submitting a change, also run:

```sh
tools/assemble-display --check
cargo fmt -p libertas-tax -- --check
cargo clippy --locked --all-targets -- -D warnings
```

To run the App on your own Libertas Hub, open this directory in VS Code with
Libertas Studio, register your Hub, and deploy the project as described in
[Using Libertas Studio](https://github.com/LibertasIoT/libertas-doc/blob/main/guides/studio.md#deploying-to-a-hub).
The Hub builds the Rust source natively after submission. Then create a task for
the App to start the interview, as described in
[configuring and using the deployed App](https://github.com/LibertasIoT/libertas-doc/blob/main/guides/studio.md#configure-and-use-the-deployed-app).
Use synthetic data only: the App is a prototype and its estimates are not tax
advice.

## History

This repository starts from a source snapshot of the App's development
history, which was itself extracted from `libertas-samples` without importing
other Apps. Coverage references can cite that earlier development history;
CODE-METRICS.md identifies its measured source revision.
