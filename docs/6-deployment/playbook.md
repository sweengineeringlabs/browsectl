# Release playbook

How to release `browse`, and how to take a bad release back. Failures are in the
[runbook](../7-operations/runbook.md).

## What is released

A GitHub release tagged `v<version>`:

| File | Purpose |
|---|---|
| `browsectl-<target>[.exe]` | the `browse` binary for each of four platforms: `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc` |
| `SHA256SUMS` | checksum of every binary |
| `toolchain-manifest.json`, `browsectl-toolchain-manifest.json` | what `justjs toolchain install` reads: tool, version, the 40-character source commit and each binary's SHA-256 |

A platform missing from the manifest cannot be installed by `justjs`, so a complete release has
all four; the release workflow and `scripts/dev/ship.sh` refuse to produce a manifest without
them. The executable is `browse`; the asset is named `browsectl-<target>` after the tool.

## Releasing

Run from a clean checkout of `main`.

1. **Bump the version.** `version` in `scm/Cargo.toml`, the `browsectl` dependency's `version`
   in `scm/main/bin/Cargo.toml`, and the two `version` lines for `browsectl` and `browsectl-bin`
   in `scm/Cargo.lock`. Add a dated section to `CHANGELOG.md` listing only what the tag will
   contain. Open a pull request and wait for CI.
2. **Verify locally** (this works without GitHub Actions):

   ```sh
   bash scripts/dev/build.sh all     # pins, fmt, clippy, tests, release build, verify
   ```

3. **Merge, then tag that exact commit and push the tag.** The release workflow refuses a tag
   that does not equal `v<version>`:

   ```sh
   git checkout main && git pull
   git tag -a vX.Y.Z -m vX.Y.Z
   git push origin vX.Y.Z
   ```

   A pushed tag is never moved or deleted once anyone may have fetched it. If it is wrong,
   release the next version (`v0.6.2` was tagged and never released; the first release after it
   is `v0.6.3`).
4. **Let the workflow build the draft.** `release.yml` builds on all four platforms, runs each
   binary's `--version` on its own platform, and creates a **draft** release with the files
   above. Watch it: `gh run watch`. If Actions cannot run, build the draft by hand instead
   (`bash scripts/dev/ship.sh --execute`; `--collect` adds binaries built on other machines; see
   [`scripts/dev/README.md`](../../scripts/dev/README.md)).
5. **Verify the draft before it is public.** Optionally run a downloaded binary yourself:

   ```sh
   gh release download vX.Y.Z -p 'browsectl-x86_64-unknown-linux-gnu' -p SHA256SUMS
   sha256sum -c --ignore-missing SHA256SUMS && chmod +x browsectl-x86_64-unknown-linux-gnu && ./browsectl-x86_64-unknown-linux-gnu --version
   bash scripts/dev/ship.sh --publish      # re-verifies names, checksums, manifest and commit,
                                           # runs the host binary, then asks you to type the tag;
                                           # type anything else to stop here
   ```

6. **Publish** by typing the tag at that prompt. Publishing is public and cannot be fully undone,
   so do it only when step 5 passed.

### Rolling back

| Situation | Action |
|---|---|
| Draft is wrong | `gh release delete vX.Y.Z --yes` (the tag stays), fix, create the draft again |
| Published release is bad | `gh release edit vX.Y.Z --draft=true` to hide it, or delete it; release a fixed **new** version. People who already downloaded it keep their copy |
| Tag is on the wrong commit | Do not move it. Release the next version |

## Who consumes a release

`justjs` projects pin `[browsectl]` (optional) by exact version and commit in
`justjs.config.toml`. After publishing, update pins with
`justjs toolchain pin browsectl <version> <revision>`; the revision is in the release's
`toolchain-manifest.json`. The order across repositories is: tools first (this repository),
then the `justjs` CLI in `justweb-dev`, then JustJS; see the `justweb-dev` playbook.

## Browser-dependent tests

Most tests need no browser. The tests that drive a real browser are `#[ignore]`d and do not run
in CI. Run them before a release if you changed the client or a command:

```sh
cd scm
cargo test --locked --workspace -- --ignored --test-threads=1
```

They need a Chromium-based browser (`CHROME_PATH` selects one) and take a few minutes.
