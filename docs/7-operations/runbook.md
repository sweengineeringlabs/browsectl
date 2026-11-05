# Runbook

How to recognise and fix the failures that have actually happened while building, testing and
using `browse`. Each entry has the symptom, the cause, the fix and how to check the fix worked.
The release steps are in the [release playbook](../6-deployment/playbook.md); the command
reference is `browse --help` and [`scm/README.md`](../../scm/README.md).

| Symptom | Section |
|---|---|
| `CHROME_PATH is set to … but that path does not exist` (exit 4) | [Browser not found](#browser-not-found-exit-4) |
| `HTTP GET http://localhost:<port>/json failed` (exit 4) | [Cannot connect to a port](#cannot-connect-to-a-port-exit-4) |
| `--url is required`, `unknown option: …`, `invalid --max-age value` (exit 2) | [Usage errors](#usage-errors-exit-2) |
| `condition not met within Ns` (exit 3) | [`wait` timed out](#wait-timed-out-exit-3) |
| Browsers left running after a script died | [Orphaned browsers](#orphaned-browsers) |
| A CI job is cancelled, not failed | [macOS or other job cancelled](#a-ci-job-is-cancelled-not-failed) |
| `Blocking waiting for file lock on build directory` | [Cargo waits on the build lock](#cargo-waits-on-the-build-lock) |
| Linker error `LNK1318`, `os error 112`, or inexplicable test failures | [Disk full](#disk-full) |
| `$'\r': command not found` running a script | [Windows line endings in a script](#windows-line-endings-in-a-script) |
| The help snapshot test fails | [Help snapshot test fails](#help-snapshot-test-fails) |
| `toolchain pins` or `docs` is red | [A consistency check fails](#a-consistency-check-fails) |

The messages below are those printed by the published `browse` 0.6.4.

## Browser not found (exit 4)

**Symptom.** `Error: CHROME_PATH is set to '<path>' but that path does not exist`, exit code 4.
With no `CHROME_PATH` and no browser installed the message is
`No Chromium-based browser found. Install Chrome/Edge/Brave or set CHROME_PATH.`

**Cause.** `launch` looks for Chrome, Chromium, Edge or Brave in the usual places, or uses
`CHROME_PATH`. A `CHROME_PATH` that points nowhere is an error, not a fallback to searching.

**Fix.** Install a Chromium-based browser, or point `CHROME_PATH` at an existing executable (or
unset it):

```sh
CHROME_PATH="/opt/chromium/chrome" browse launch --url https://example.com --port 9222
```

**Check.** `browse launch` prints `Browser launched.` and exits 0.

## Cannot connect to a port (exit 4)

**Symptom.** `Error: HTTP GET http://localhost:<port>/json failed (exit 28)` (or `exit 7`), exit
code 4, from any command that attaches (`eval`, `wait`, `screenshot`, `stop`, …).

**Cause.** Nothing answered on `--port` (default 9222): no browser was launched there, it was
stopped, or it is on a different port. `28` is curl timing out and `7` a refused connection.
`curl` must be on `PATH`, because the debugger URL is found with it.

**Fix.** Launch a browser on that port, or use the port you launched with:

```sh
browse launch --url https://example.com --port 9222
browse eval --port 9222 --script "document.title"
```

**Check.** `curl http://localhost:9222/json` prints a JSON list, and the command no longer
exits 4.

## Usage errors (exit 2)

**Symptom.** `Error: --url is required`, `Error: unknown option: --bogus`,
`Error: invalid --max-age value: 'soon' (expected e.g. 30, 30s, 5m, 1h)`, or a usage message
after an unknown command; exit code 2.

**Cause.** A required option is missing, an option does not belong to the command, or a value
cannot be parsed.

**Fix.** Read the command's own help, which lists every option, an example and the exit codes:

```sh
browse launch --help
```

**Check.** The command runs. (`--help` itself exits 0; an unknown command lists the valid ones.)

## `wait` timed out (exit 3)

**Symptom.** `Error: condition not met within 2s`, exit code 3.

**Cause.** The selector, text or navigation condition did not become true within `--timeout`
(default 30 seconds, whole seconds). `wait` polls every 100 ms.

**Fix.** Check the selector against the real page (`browse get-dom`), raise `--timeout`, or wait
for the right condition (`--navigation`, `--text`, `--selector`).

**Check.** `Condition met.` and exit 0.

## Orphaned browsers

**Symptom.** Headless browsers keep running after the script or CI job that launched them
died, holding their debug ports and memory.

**Cause.** `launch` leaves the browser running by design; it is stopped by `browse stop`. If the
process that called `launch` dies first, nothing stops it. `launch` records each session so
`reap` can find those (RFC-0003).

**Fix.** List what would be cleaned up, then clean it:

```sh
browse reap --dry-run
# Would reap session on port 9333 (caller process is no longer running)
# Found 1 session(s).
browse reap                     # close them and delete their records
browse reap --max-age 1h        # also close sessions older than an hour even if their caller lives
browse stop --port 9333         # or stop exactly one
```

`browse launch --reap-stale` does a caller-liveness reap before launching. Session records are
under `<temp dir>/browsectl/sessions` unless `BROWSECTL_SESSION_DIR` is set.

**Check.** `browse reap --dry-run` prints `No orphaned or stale sessions found.`

## A CI job is cancelled, not failed

**Symptom.** A `macos-15` or `macos-15-intel` job ends *cancelled*, with *"The job was not
acquired by Runner of type hosted"* or a very long queue.

**Cause.** GitHub had no macOS runner capacity. It is not a test failure.

**Fix.** Re-run the job. A job that ran and **failed** is a real failure and must be
investigated. (This repository is public, so its Actions run normally; a run that fails within
seconds with no steps would be an account-level problem, not code.)

**Check.** The re-run job executes its steps.

## Cargo waits on the build lock

**Symptom.** `Blocking waiting for file lock on build directory` and nothing else for minutes.

**Cause.** Another cargo process (an editor's rust-analyzer, a second terminal, a background
agent) holds the lock on the shared `scm/target/` directory.

**Fix.** Do not wait. Build into an isolated directory with two parallel jobs:

```sh
bash scripts/dev/build.sh test
```

`BROWSECTL_TARGET_DIR` changes the directory (default `.work/target`, git-ignored) and
`BROWSECTL_JOBS` the parallelism.

**Check.** The script completes without the lock message.

## Disk full

**Symptom.** On Windows: `LINK : fatal error LNK1318: Unexpected PDB error`,
`There is not enough space on the disk. (os error 112)`, or tests failing that pass on a retry.
On Unix: `No space left on device`.

**Cause.** The disk is full. A workspace build with its test binaries needs a gigabyte or more.

**Fix.** Delete build output you can recreate; never delete source or `.git`:

```sh
rm -rf .work/target scm/target
```

**Check.** Several gigabytes are free and `bash scripts/dev/build.sh test` passes. Do not trust a
test result obtained while the disk was full.

## Windows line endings in a script

**Symptom.** Running a `.sh` file fails with `$'\r': command not found` or
`bash\r: No such file or directory`.

**Cause.** Git converted the script to CRLF on checkout (`core.autocrlf=true`).

**Fix.** `.gitattributes` marks `*.sh` as `text eol=lf`, so a fresh checkout is correct. Rewrite
an already-converted file from the repository:

```sh
rm scripts/dev/build.sh && git checkout -- scripts/dev/build.sh
```

**Check.** `file scripts/dev/build.sh` reports no `CRLF`.

## Help snapshot test fails

**Symptom.** `test_help_every_command_matches_its_snapshot` (or the top-level one) fails with
`` `browse <command> --help` help changed; if intended, regenerate… ``.

**Cause.** A help text changed and its snapshot did not. That is the test doing its job.

**Fix.** If the change is intended, regenerate and review the diff:

```sh
BROWSE_UPDATE_SNAPSHOTS=1 cargo test -p browsectl-bin --test help_snapshot_test
git diff scm/main/bin/tests/snapshots
```

**Check.** The test passes without the variable and the diff shows only the wording you meant to
change.

## A consistency check fails

- **`toolchain pins` red.** A workflow's `dtolnay/rust-toolchain@…` pin or
  `scm/rust-toolchain.toml` differs from `rust-version` in `scm/Cargo.toml`. Make them equal in
  one change.
- **`docs` red.** `bash scripts/dev/check-docs.sh` names the file: a relative link that no
  longer resolves, or a three-part `Rust X.Y.Z` that is not the declared `rust-version`. The
  RFCs are exempt from the version check because they are point-in-time proposals.

**Check.** `bash scripts/dev/check-docs.sh` and `bash scripts/dev/build.sh pins` pass.
