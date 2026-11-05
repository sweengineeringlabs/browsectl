#!/usr/bin/env bash
# Check the documentation for two kinds of rot:
#
#   1. Broken relative links in README.md, scm/README.md, scm/main/bin/README.md,
#      scripts/dev/README.md and docs/.
#      A link is `[text](target)`; http(s)/mailto/#anchor targets are skipped and
#      fenced code blocks are ignored. A fragment (`file.md#section`) is checked
#      only for the file, not the heading.
#   2. A Rust version that differs from `rust-version` in scm/Cargo.toml. Every
#      three-part "Rust X.Y.Z" must equal it; a two-part floor ("Rust 1.88") is allowed.
#
# RFCs are point-in-time proposals (see docs/rfcs/README.md), so their version numbers are
# history and are not checked for (2); their links are still checked.
#
# Usage: scripts/dev/check-docs.sh        (run from anywhere; exits 1 on a finding)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

CARGO_TOML="scm/Cargo.toml"
EXEMPT_PREFIXES=("docs/rfcs/")

fail=0
note() { echo "error: $*" >&2; fail=1; }

declared="$(sed -n 's/^rust-version *= *"\([0-9][0-9.]*\)".*/\1/p' "${CARGO_TOML}" | head -1)"
[ -n "${declared}" ] || { echo "error: no rust-version in ${CARGO_TOML}" >&2; exit 1; }

files=()
for f in README.md CONTRIBUTING.md scm/README.md scm/main/bin/README.md scripts/dev/README.md; do [ -f "${f}" ] && files+=("${f}"); done
while IFS= read -r f; do files+=("${f}"); done < <(find docs -name '*.md' 2>/dev/null | sort)
[ "${#files[@]}" -gt 0 ] || { echo "error: no documentation files found" >&2; exit 1; }

is_exempt() {
  local f="$1" p
  for p in ${EXEMPT_PREFIXES[@]+"${EXEMPT_PREFIXES[@]}"}; do [[ "${f}" == "${p}"* ]] && return 0; done
  return 1
}

# Markdown with fenced code blocks removed, so examples are not treated as links.
strip_fences() { awk '/^[[:space:]]*```/ { skip = !skip; next } !skip { print }' "$1"; }

for file in "${files[@]}"; do
  dir="$(dirname "${file}")"

  # 1. relative links (checked everywhere, including exempt files)
  while IFS= read -r target; do
    case "${target}" in
      ''|http://*|https://*|mailto:*|\#*) continue ;;
    esac
    path="${target%%#*}"; path="${path%% *}"
    [ -n "${path}" ] || continue
    if [ ! -e "${dir}/${path}" ]; then
      note "${file}: broken link '${target}' (no such file: ${dir}/${path})"
    fi
  done < <(strip_fences "${file}" | tr -d '\r' | grep -oE '\]\([^)]+\)' | sed -E 's/^\]\((.*)\)$/\1/' || true)

  is_exempt "${file}" && continue

  # 2. Rust versions
  while IFS= read -r found; do
    version="${found##* }"
    [ "${version}" = "${declared}" ] || note "${file}: says '${found}' but rust-version is ${declared}"
  done < <(strip_fences "${file}" | tr -d '\r' | grep -oE 'Rust [0-9]+\.[0-9]+\.[0-9]+' || true)
done

if [ "${fail}" -ne 0 ]; then
  echo "docs check FAILED" >&2
  exit 1
fi
echo "docs check OK: ${#files[@]} files, relative links resolve, Rust versions equal ${declared}"
