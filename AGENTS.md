# AGENTS.md

This repository holds the PitCrew plugin API, the developer tool `pcplugin`, the registry and the
official plugins. Read [README.md](./README.md) and [docs/](./docs/) before you change anything.

## How agents write: 80% ASD-STE100

Everything an LLM agent writes for a person follows ASD-STE100 Simplified Technical English at
least 80% of the time. This includes replies, issue and PR text, commit messages and docs.

- Use short sentences: at most 20 words for an instruction and 25 for a description.
- Write one instruction per sentence, in the active voice, with the verb first.
- Use one word for one meaning, and the same term for the same thing every time.
- Use simple tenses. Do not use idioms, slang or ornamental words.
- Use lists and tables for steps and comparisons, not long paragraphs.
- The other 20% is room for code identifiers, product names, quotes and nuance that STE cannot
  express without loss of meaning.

## Rules for this repository

- The WIT in `wit/` is a contract with every published plugin. Inside `@1`, you can only add.
  A removal or a change is `@2`, with an ADR in pitcrew-dev.
- Keep plugin logic behind the `Host` trait, so `cargo test` runs it natively. Only `lib.rs` touches
  the generated bindings.
- Gates, the same as CI (`.github/workflows/ci.yml`):
  1. `cargo test -p pitflag-wire -p pitcrew-flags -p pcplugin`
  2. `cargo build --release --target wasm32-wasip2 -p pitcrew-flags -p hello-plugin`
  3. `pcplugin validate` on every folder in `plugins/` and `examples/`
  4. `pcplugin run … --seconds 6` on each plugin
  Report the real exit codes. Do not pipe the cargo output.
- Docs, UI text and code are en-US.
