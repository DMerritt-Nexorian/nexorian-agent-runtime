# Nexorian Agent Runtime

Lightweight agent runtime with a `no_std` core and an optional standard-library HTTP adapter for local Ollama inference.

## Requirements

- Rust stable toolchain
- [Ollama](https://ollama.com) running locally
- Llama 3.2 pulled with `ollama pull llama3.2`

## Start local inference

```sh
ollama pull llama3.2
ollama serve
```

In another terminal:

```sh
cargo run --example ollama_bridge
```

The bridge sends a non-streaming JSON request to `http://127.0.0.1:11434/api/generate`. The HTTP adapter defaults to a 30-second request timeout. Configure another endpoint/model through `LocalHttpEngine::new(endpoint, model)`, or set a different timeout with `with_timeout_ms(milliseconds)`.

## Verify live inference

```sh
OLLAMA_MODEL=llama3.2 cargo run --example ollama_smoke
```

The smoke example makes an actual generation request and fails on transport, HTTP, JSON, or empty-response errors.

## Rust quality gates

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check --all-targets --all-features
cargo test --all-targets --all-features
```

The CI workflow runs these gates and a separate job that installs Ollama, pulls Llama 3.2, starts the server, and executes the live inference smoke example.

## Preserve the no_std core

The default `std` feature enables the reqwest-based local HTTP adapter. To build the core without the HTTP adapter and its dependencies:

```sh
cargo check --no-default-features
```

## Registered tools

- content_create
- write_draft
- language_practice
- task_break
- feedback_analyze
- contract_review
- code_audit
- proposal_write
- meeting_summary
- support_reply

**Important:** the current tool implementations are deterministic scaffolds returning structured status/results. They are not yet connected to their respective product backends and should not be represented as completed production services.
