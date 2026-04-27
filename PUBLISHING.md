# Publishing

This crate is published to [crates.io](https://crates.io/crates/openruntimes-types-for-rust).

## Setup

1. Create a crates.io account at https://crates.io and verify the email on your account
2. Generate an API token at https://crates.io/settings/tokens
   - Scope: `publish-update` (sufficient to publish new versions of an existing crate)
3. Add the token as a GitHub Actions secret:
   - Secret name: `CARGO_REGISTRY_TOKEN`
   - Secret value: the generated token

## Publishing

Publishing is automated via GitHub Actions. To publish a new version:

1. Create a new GitHub release with a tag matching the desired version (e.g., `1.0.0` or `v1.0.0`)
2. The workflow will automatically update `Cargo.toml` and publish to crates.io

## Manual Publishing

```bash
cargo login
cargo publish
```
