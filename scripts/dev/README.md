# scripts/dev

Build, verify and ship browsectl from your own machine. Nothing here needs GitHub Actions.

| Script | Purpose | Changes anything remote? |
|---|---|---|
| `build.sh` | Run the CI checks, build the release binary for this machine, verify it | No |
| `ship.sh` | Build, verify and create a GitHub release; make it public | Only with `--execute` / `--publish` |

Both need bash (on Windows, Git Bash), a Rust toolchain matching `scm/Cargo.toml`, and Python 3. `ship.sh --execute` and `--publish` also need the GitHub CLI (`gh auth login`).

## build.sh

```
scripts/dev/build.sh [pins|fmt|clippy|test|build|release|verify|all]
```

`all` (the default) runs, in order: `pins` (compiler pins agree across `scm/Cargo.toml`, `scm/rust-toolchain.toml` and the workflows), `fmt`, `clippy -D warnings`, `test`, `release` (build for this machine, check `--version` prints `browse <version> (browsectl)`, stage the files) and `verify` (re-check the staged files independently).

Output goes to `.work/` (git-ignored), so it never contends with other cargo builds for a lock.

| Variable | Default | Meaning |
|---|---|---|
| `BROWSECTL_TARGET_DIR` | `.work/target` | cargo target directory |
| `BROWSECTL_JOBS` | `2` | cargo parallelism (keeps memory low) |
| `BROWSECTL_DIST_DIR` | `.work/dist` | where the release is staged |


Only your own platform can be built locally. A complete release has four: `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`.

## Releasing

The normal route is the release workflow (`.github/workflows/release.yml`): push a `v*` tag and it builds all four platforms and creates a **draft** release. Use `ship.sh` when the workflow cannot run (for example GitHub Actions is unavailable or billing is blocked), or to re-check a draft the workflow made.

### Checklist

1. Bump the version in `scm/Cargo.toml` (and `Cargo.lock`, and the `version =` of the `browsectl` dependency in `scm/main/bin/Cargo.toml`). Add a dated section to `CHANGELOG.md`. Merge it.
2. Check out the merge commit on `main`, clean tree: `git status` shows nothing.
3. Tag that exact commit and push the tag:
   ```
   git tag -a vX.Y.Z -m vX.Y.Z
   git push origin vX.Y.Z
   ```
   A tag must never be moved once pushed. If it is wrong, bump the version instead.
4. Either let the workflow build the draft, or do it by hand (below).
5. Re-verify and publish: `scripts/dev/ship.sh --publish`.

### Shipping by hand

```
scripts/dev/ship.sh [--collect DIR]... [--allow-partial] [--execute]
scripts/dev/ship.sh --publish [--allow-partial] [--yes]
```

1. **Dry run** (no flags): runs every check, builds this machine's binary, verifies the release exactly as it would be published, and prints its contents. Changes nothing remote.
2. **`--execute`**: also creates a **draft** release and downloads it again to confirm the upload is byte-identical to what was verified. A draft is invisible to the public.
3. **`--publish`**: downloads the draft, verifies it again, checks it was built from the tagged commit, then asks you to type the tag name. Only then is it made public and marked Latest. `--yes` skips the prompt (for scripts only).

To include platforms you cannot build here, build on each other machine with `scripts/dev/build.sh release`, copy its `.work/dist` folder **and** the `.work/dist.build-info.json` beside it, and pass the folder with `--collect DIR` (repeat per machine). The version, commit and checksums are checked before anything is added; each platform may come from only one machine. Without all four, pass `--allow-partial`.

### What it refuses

- a working tree with uncommitted changes;
- a tag that does not exist, is not at `HEAD`, or (`--execute`) is not on the remote;
- a release that already exists (`--execute`), or one that is already public (`--publish`);
- a missing platform (unless `--allow-partial`);
- any checksum, name, version or manifest mismatch, including in a draft someone edited;
- an unconfirmed `--publish`.

Each refusal prints what is wrong and how to fix it.

### Release contents

`browsectl-<target>` (one raw binary per platform, `.exe` on Windows), `SHA256SUMS`, and `toolchain-manifest.json` plus `browsectl-toolchain-manifest.json` (`schema`, `tool`, `version`, `revision`, and each asset's target, file and sha256). `ship.sh` will not build a manifest unless all four binaries are present. The executable is `browse`; assets are named `browsectl-<target>`.

## Recovery

| Situation | Do this |
|---|---|
| Draft is wrong (bad asset, wrong platform set) | `gh release delete vX.Y.Z` (the tag stays), fix, run `--execute` again |
| Tag points at the wrong commit | Do not move it. Bump the version, tag the new commit |
| Published a bad release | `gh release edit vX.Y.Z --draft=true` to hide it, or delete it; ship a fixed version. Anyone who already downloaded it keeps their copy |
| `ship.sh` says the tree is dirty | Commit or stash. The build must be reproducible from the tag |
| `--collect` says a binary changed after it was built | Rebuild on that machine and copy it again |

