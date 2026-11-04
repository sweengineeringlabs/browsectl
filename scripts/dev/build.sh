#!/usr/bin/env bash
# Build and verify browsectl on this machine, with no dependency on GitHub Actions.
#
# Runs the same checks as .github/workflows (toolchain pins, fmt, clippy, tests)
# and rehearses the release: builds the release binary for THIS machine's
# target, asserts its --version, stages it with SHA256SUMS and a toolchain
# manifest, and verifies that staging independently.
#
# Usage: scripts/dev/build.sh [pins|fmt|clippy|test|build|release|verify|all]
#   all (default) = pins, fmt, clippy, test, release, verify
#
# Environment:
#   BROWSECTL_TARGET_DIR  isolated cargo target dir (default: <repo>/.work/target)
#   BROWSECTL_JOBS        cargo parallelism (default: 2; keeps peak memory low)
#   BROWSECTL_DIST_DIR    release staging dir (default: <repo>/.work/dist)
#
# Limits: only this machine's platform can be built here. A complete release
# (all four platforms) is produced by the release workflow, or by combining
# builds from several machines with scripts/dev/ship.sh --collect.
#
# This file is also sourced by scripts/dev/ship.sh; it only runs a step when
# executed directly.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORKSPACE_DIR="${REPO_ROOT}/scm"
CARGO_TOML="${REPO_ROOT}/scm/Cargo.toml"
TOOLCHAIN_FILE="scm/rust-toolchain.toml"   # relative to the repo root; empty when the repo has none
TOOL="browsectl"                       # name used in the release manifest and asset names
PACKAGE="browsectl-bin"                 # cargo package that builds the executable
BIN_NAME="browse"                    # executable cargo produces
VERSION_FMT='browse %s (browsectl)'         # printf format of `<exe> --version`; %s is the version
PIN_STYLE="uses"             # how workflows pin the compiler: uses | toolchain
HAS_MANIFEST="1"           # 1 = release carries toolchain-manifest.json; 0 = SHA256SUMS only
EXTRA_FILES=""         # files (relative to the repo root) shipped beside the binaries
# Platforms a complete release must contain, in manifest order.
RELEASE_TARGETS="x86_64-unknown-linux-gnu aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-msvc"
DEFAULT_JOBS=2
PYTHON=""

die() { echo "error: $*" >&2; exit 1; }

init_environment() {
  export CARGO_TARGET_DIR="${BROWSECTL_TARGET_DIR:-${REPO_ROOT}/.work/target}"
  JOBS="${BROWSECTL_JOBS:-${DEFAULT_JOBS}}"
  DIST_DIR="${BROWSECTL_DIST_DIR:-${REPO_ROOT}/.work/dist}"
  [[ "${JOBS}" =~ ^[1-9][0-9]*$ ]] || die "BROWSECTL_JOBS must be a positive integer, got '${JOBS}'."
  # `python3` can be a non-working Windows Store stub, so test that it runs.
  local candidate
  for candidate in python3 python; do
    if "${candidate}" -c 'import sys' >/dev/null 2>&1; then PYTHON="${candidate}"; break; fi
  done
  [ -n "${PYTHON}" ] || die "no working python3 or python on PATH (needed for manifests and checksums)."
}

declared_rust_version() { grep -m1 '^rust-version' "${CARGO_TOML}" | cut -d'"' -f2; }

step_pins() {
  echo "==> pins"
  local declared channel pins expected active
  declared="$(declared_rust_version)"
  [ -n "${declared}" ] || die "rust-version not found in ${CARGO_TOML}"
  echo "declared rust-version: ${declared}"
  if [ -n "${TOOLCHAIN_FILE}" ]; then
    channel="$(grep -m1 '^channel' "${REPO_ROOT}/${TOOLCHAIN_FILE}" | cut -d'"' -f2)"
    echo "${TOOLCHAIN_FILE} channel: ${channel}"
    [ "${channel}" = "${declared}" ] || die "${TOOLCHAIN_FILE} (${channel}) differs from rust-version (${declared})"
  fi
  case "${PIN_STYLE}" in
    uses)
      pins="$(grep -h -o -E 'uses: dtolnay/rust-toolchain@[0-9A-Za-z.]+' "${REPO_ROOT}"/.github/workflows/*.yml | sed 's/^uses: //' | sort -u)"
      expected="dtolnay/rust-toolchain@${declared}" ;;
    toolchain)
      pins="$(grep -h -o -E 'toolchain: "[0-9][0-9.]*"' "${REPO_ROOT}"/.github/workflows/*.yml | sort -u)"
      expected="toolchain: \"${declared}\"" ;;
    *) die "unknown PIN_STYLE '${PIN_STYLE}'" ;;
  esac
  echo "${pins}"
  [ -n "${pins}" ] || die "no toolchain pins found in .github/workflows"
  [ "${pins}" = "${expected}" ] || die "workflow toolchain pins differ from rust-version ${declared}"
  active="$(cd "${WORKSPACE_DIR}" && rustc --version | awk '{print $2}')"
  [ "${active}" = "${declared}" ] || die "the active compiler is ${active}, not ${declared}; run: rustup toolchain install ${declared}"
  echo "==> pins: OK"
}

step_fmt() {
  echo "==> fmt"
  (cd "${WORKSPACE_DIR}" && cargo fmt --all --check)
  echo "==> fmt: OK"
}

step_clippy() {
  echo "==> clippy"
  (cd "${WORKSPACE_DIR}" && cargo clippy --workspace --all-targets --locked -j "${JOBS}" -- -D warnings)
  echo "==> clippy: OK"
}

step_test() {
  echo "==> test"
  (cd "${WORKSPACE_DIR}" && cargo test --workspace --locked -j "${JOBS}")
  echo "==> test: OK"
}

step_build() {
  echo "==> build (debug)"
  (cd "${WORKSPACE_DIR}" && cargo build -p "${PACKAGE}"  --locked -j "${JOBS}")
  echo "==> build: OK"
}

host_target() { rustc -vV | sed -n 's/^host: //p'; }

crate_version() {
  (cd "${WORKSPACE_DIR}" && cargo metadata --no-deps --format-version 1) | "${PYTHON}" -c '
import json, sys
package = sys.argv[1]
print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"] == package))' "${PACKAGE}"
}

# Write SHA256SUMS and both manifest files for every asset in DIR.
write_checksums_and_manifest() {
  local dir="$1" version="$2" revision="$3"
  "${PYTHON}" - "${dir}" "${TOOL}" "${version}" "${revision}" "${RELEASE_TARGETS}" "${HAS_MANIFEST}" <<'PY'
import hashlib, json, os, sys
directory, tool, version, revision, order, has_manifest = sys.argv[1:7]
order = order.split()
assets = []
for name in sorted(os.listdir(directory)):
    if not name.startswith(tool + "-") or name.endswith(".json") or name == "SHA256SUMS":
        continue
    target = name[len(tool) + 1:]
    if target.endswith(".exe"):
        target = target[:-4]
    if target not in order:
        sys.exit(f"unexpected asset {name!r}: {target!r} is not a release target")
    digest = hashlib.sha256(open(os.path.join(directory, name), "rb").read()).hexdigest()
    assets.append({"target": target, "file": name, "sha256": digest})
if not assets:
    sys.exit(f"no release assets found in {directory}")
assets.sort(key=lambda a: order.index(a["target"]))
if has_manifest == "1":
    manifest = {"schema": 1, "tool": tool, "version": version, "revision": revision, "assets": assets}
    for name in ("toolchain-manifest.json", f"{tool}-toolchain-manifest.json"):
        with open(os.path.join(directory, name), "w") as f:
            f.write(json.dumps(manifest, indent=2) + "\n")
with open(os.path.join(directory, "SHA256SUMS"), "w") as f:
    for a in assets:
        f.write(f"{a['sha256']}  {a['file']}\n")
PY
}

# Independently verify a staged or downloaded release directory.
# MODE is "all" (every release target must be present) or "any".
verify_dist() {
  local dir="$1" mode="${2:-any}"
  [ -d "${dir}" ] || die "nothing to verify: ${dir} does not exist"
  "${PYTHON}" - "${dir}" "${TOOL}" "${mode}" "${RELEASE_TARGETS}" "${HAS_MANIFEST}" "${EXTRA_FILES}" <<'PY'
import hashlib, json, os, re, sys
directory, tool, mode, order, has_manifest, extras = sys.argv[1:7]
order = order.split()
extras = [os.path.basename(e) for e in extras.split()]
errors = []
def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()
def target_of(file):
    t = file[len(tool) + 1:]
    return t[:-4] if t.endswith(".exe") else t
names = ("toolchain-manifest.json", f"{tool}-toolchain-manifest.json") if has_manifest == "1" else ()
sums = {}
sums_path = os.path.join(directory, "SHA256SUMS")
if not os.path.isfile(sums_path):
    errors.append("missing SHA256SUMS")
else:
    for line in open(sums_path).read().splitlines():
        digest, _, name = line.partition("  ")
        sums[name.lstrip("*")] = digest
assets = []
if has_manifest == "1":
    missing = [n for n in names if not os.path.isfile(os.path.join(directory, n))]
    if missing:
        sys.exit("verification failed:\n  " + "\n  ".join(f"missing {n}" for n in missing))
    text = [open(os.path.join(directory, n)).read() for n in names]
    if text[0] != text[1]:
        errors.append("the two manifest files differ")
    m = json.loads(text[0])
    if m.get("schema") != 1: errors.append(f"schema is {m.get('schema')!r}, expected 1")
    if m.get("tool") != tool: errors.append(f"manifest tool is {m.get('tool')!r}, expected {tool!r}")
    if not re.fullmatch(r"[0-9A-Za-z.+-]+", str(m.get("version", ""))): errors.append(f"bad version {m.get('version')!r}")
    if not re.fullmatch(r"[0-9a-f]{40}", str(m.get("revision", ""))): errors.append(f"revision {m.get('revision')!r} is not a 40-character commit SHA")
    assets = [{"target": a.get("target"), "file": a.get("file"), "sha256": a.get("sha256")} for a in (m.get("assets") or [])]
else:
    for name in sorted(os.listdir(directory)):
        if name.startswith(tool + "-") and not name.endswith(".json"):
            assets.append({"target": target_of(name), "file": name, "sha256": sums.get(name)})
if not assets: errors.append("the release lists no assets")
seen = set()
for a in assets:
    target, file, digest = a["target"], a["file"], a["sha256"]
    if target not in order: errors.append(f"unknown target {target!r}")
    if target in seen: errors.append(f"duplicate target {target!r}")
    seen.add(target)
    expected = f"{tool}-{target}" + (".exe" if target and "windows" in target else "")
    if file != expected: errors.append(f"asset for {target} is named {file!r}, expected {expected!r}")
    path = os.path.join(directory, file or "")
    if not os.path.isfile(path):
        errors.append(f"missing asset {file}")
        continue
    actual = sha(path)
    if digest is not None and actual != digest: errors.append(f"{file}: sha256 {actual} does not match the recorded checksum ({digest})")
    if sums.get(file) != actual: errors.append(f"{file}: the SHA256SUMS entry does not match the file")
extra_sums = set(sums) - {a["file"] for a in assets}
if extra_sums: errors.append(f"SHA256SUMS lists files that are not release assets: {sorted(extra_sums)}")
for e in extras:
    if not os.path.isfile(os.path.join(directory, e)): errors.append(f"missing {e}")
allowed = {a["file"] for a in assets} | set(names) | {"SHA256SUMS"} | set(extras)
extra_files = set(os.listdir(directory)) - allowed
if extra_files: errors.append(f"unexpected files in the release: {sorted(extra_files)}")
if mode == "all":
    absent = [t for t in order if t not in seen]
    if absent: errors.append(f"the release is missing platforms: {absent}")
if errors:
    sys.exit("verification failed:\n  " + "\n  ".join(errors))
print(f"verified {len(assets)} asset(s) for {tool}: {sorted(seen)}")
PY
  verify_host_binary "${dir}"
}

# Run the asset built for this machine, if the release contains one.
verify_host_binary() {
  local dir="$1" host suffix version asset expected out
  host="$(host_target)"
  suffix=""; case "${host}" in *windows*) suffix=".exe" ;; esac
  asset="${dir}/${TOOL}-${host}${suffix}"
  if [ ! -f "${asset}" ]; then
    echo "note: no asset for this machine (${host}); its binary was not executed"
    return 0
  fi
  if [ "${HAS_MANIFEST}" = 1 ]; then
    version="$("${PYTHON}" -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "${dir}/toolchain-manifest.json")"
  else
    version="$(crate_version)"
  fi
  # shellcheck disable=SC2059
  expected="$(printf "${VERSION_FMT}" "${version}")"
  chmod +x "${asset}" 2>/dev/null || true
  out="$("${asset}" --version)"
  [ "${out}" = "${expected}" ] || die "${asset##*/} reports '${out}', expected '${expected}'"
  echo "ran ${asset##*/}: ${out}"
}

# Record what this machine built, beside (not inside) the release directory, so a
# build from another machine can be combined with `ship.sh --collect`.
write_build_info() {
  local dir="$1" version="$2" revision="$3" target="$4" asset="$5"
  "${PYTHON}" - "${dir}" "${TOOL}" "${version}" "${revision}" "${target}" "${asset}" <<'PY'
import hashlib, json, os, sys
directory, tool, version, revision, target, asset = sys.argv[1:7]
digest = hashlib.sha256(open(os.path.join(directory, asset), "rb").read()).hexdigest()
info = {"tool": tool, "version": version, "revision": revision, "target": target,
        "assets": [{"file": asset, "sha256": digest}]}
with open(directory.rstrip("/\\") + ".build-info.json", "w") as f:
    f.write(json.dumps(info, indent=2) + "\n")
PY
}

step_release() {
  echo "==> release (this machine only)"
  local host suffix version bin out expected revision
  host="$(host_target)"; version="$(crate_version)"
  suffix=""; case "${host}" in *windows*) suffix=".exe" ;; esac
  echo "tool=${TOOL} version=${version} target=${host}"
  (cd "${WORKSPACE_DIR}" && cargo build --release --locked -j "${JOBS}" -p "${PACKAGE}"  --target "${host}")
  bin="${CARGO_TARGET_DIR}/${host}/release/${BIN_NAME}${suffix}"
  [ -f "${bin}" ] || die "expected binary not found: ${bin}"
  # shellcheck disable=SC2059
  expected="$(printf "${VERSION_FMT}" "${version}")"
  out="$("${bin}" --version)"
  [ "${out}" = "${expected}" ] || die "the binary reports '${out}', expected '${expected}'"
  rm -rf "${DIST_DIR}"; mkdir -p "${DIST_DIR}"
  cp "${bin}" "${DIST_DIR}/${TOOL}-${host}${suffix}"
  local extra
  for extra in ${EXTRA_FILES}; do cp "${REPO_ROOT}/${extra}" "${DIST_DIR}/$(basename "${extra}")"; done
  revision="$(git -C "${REPO_ROOT}" rev-parse HEAD)"
  if [ -n "$(git -C "${REPO_ROOT}" status --porcelain)" ]; then
    echo "warning: the working tree has uncommitted changes; this build is not reproducible from ${revision}" >&2
  fi
  write_checksums_and_manifest "${DIST_DIR}" "${version}" "${revision}"
  write_build_info "${DIST_DIR}" "${version}" "${revision}" "${host}" "${TOOL}-${host}${suffix}"
  echo "staged in ${DIST_DIR} (this machine's platform only; the full release has four)"
  echo "==> release: OK"
}

step_verify() {
  echo "==> verify"
  verify_dist "${DIST_DIR}" any
  echo "==> verify: OK"
}

usage() { sed -n '2,22p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; }

main() {
  init_environment
  case "${1:-all}" in
    pins)    step_pins ;;
    fmt)     step_fmt ;;
    clippy)  step_clippy ;;
    test)    step_test ;;
    build)   step_build ;;
    release) step_release ;;
    verify)  step_verify ;;
    all)     step_pins; step_fmt; step_clippy; step_test; step_release; step_verify ;;
    -h|--help|help) usage ;;
    *) usage >&2; die "unknown step '${1}'" ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi
