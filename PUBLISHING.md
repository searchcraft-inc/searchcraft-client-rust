# Publishing

Maintainer notes for releasing `searchcraft` to [crates.io](https://crates.io).
End users do not need any of this — they consume the crate with
`cargo add searchcraft`.

## Releasing

Publishing is automated. A push to `main` runs
[`.github/workflows/publish.yml`](.github/workflows/publish.yml), which:

1. Reads the version from `Cargo.toml`.
2. Asks crates.io whether that version already exists, and stops if it does.
   Merges that do not change the version are therefore a no-op, so the workflow
   can run on every merge.
3. Runs format, lint, test, and documentation checks.
4. Publishes to crates.io.
5. Tags the commit `v<version>` and opens a GitHub release, with the body taken
   from this version's `CHANGELOG.md` section.

So a release is: **bump `version` in `Cargo.toml`, add a `CHANGELOG.md` entry,
merge to `main`.** Nothing else. The tag and release are created only after the
publish succeeds, so a failed publish leaves no dangling tag.

The workflow needs one repository secret, `CARGO_REGISTRY_TOKEN`, holding a
crates.io API token with publish rights. Add it under **Settings → Secrets and
variables → Actions**.

## Manual publishing

The steps below are the same thing by hand, for a first publish or if the
workflow is unavailable.

### Prerequisites

1. Create an account on [crates.io](https://crates.io) and generate an API
   token under **Account Settings → API Tokens**.
2. Log in locally:

   ```sh
   cargo login <your-token>
   ```

You need to be listed as an owner of the crate to publish. Existing owners can
add you with `cargo owner --add <github-username>`.

### Pre-publish checklist

```sh
# Format check
cargo fmt --all --check

# Lint, including tests and examples
cargo clippy --all-targets --all-features -- -D warnings

# Tests, including doctests and the README examples
cargo test --all-features

# Documentation must build clean, since docs.rs runs this
cargo doc --no-deps --all-features

# Validate packaging without uploading
cargo publish --dry-run
```

`cargo publish --dry-run` is the important one: it packages the crate exactly as
it will be uploaded and compiles it from that package, which catches files that
are missing from the published archive but present locally.

Confirm the package contents are what you expect:

```sh
cargo package --list
```

`header.png` is excluded in `Cargo.toml` so the README banner does not ship in
every download; the README references it by absolute URL, so it still renders on
crates.io and docs.rs.

### Releasing a new version by hand

1. Update `version` in `Cargo.toml`.
2. Add a `CHANGELOG.md` entry describing the change, calling out anything
   breaking.
3. Run the pre-publish checklist above.
4. Commit, then tag and push:

   ```sh
   git tag v<version>
   git push origin main --tags
   ```

5. Publish:

   ```sh
   cargo publish
   ```

The crate appears at
[crates.io/crates/searchcraft](https://crates.io/crates/searchcraft), and
docs.rs builds the documentation automatically from the uploaded package.

## Versioning

This crate follows [Semantic Versioning](https://semver.org). While the version
is below `1.0.0`, breaking changes bump the **minor** version and everything
else bumps the patch version.

Publishes are permanent — a released version can be yanked with
`cargo yank --version <version>`, which stops new dependents from selecting it,
but it cannot be deleted or replaced. Prefer publishing a fixed patch release
over yanking.
