# Contributing

Contributions are welcome. Open an issue to discuss a substantial change, or submit a pull request for a focused improvement.

## Development checks

Before submitting a pull request, run the checks relevant to the change:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm typecheck
pnpm lint
pnpm test
pnpm build
```

The API and database can be started with `docker compose up --build`. Keep imported source files and credentials out of version control. Document any source-specific licensing or evidence limitations introduced by a change.

## Pull requests

Include a concise summary, the reason for the change, relevant test results, and any user-visible behavior changes. Update documentation when commands, routes, configuration, or data semantics change.
