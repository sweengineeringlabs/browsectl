#!/usr/bin/env bash
# Ship a browsectl release by hand, for when the release workflow cannot run
# (for example when GitHub Actions is unavailable). It builds and verifies
# locally, then creates a DRAFT GitHub release. Making the draft public is a
# separate, confirmed step. Nothing is published unless you pass --execute.
#
# Usage:
#   scripts/dev/ship.sh [--collect DIR]... [--allow-partial] [--execute]
#   scripts/dev/ship.sh --publish [--allow-partial] [--yes]
#
#   (no flags)       Run every check, build this machine's binary, verify, and
#                    print what WOULD be released. Changes nothing remote.
#   --execute        Also create the DRAFT release and re-download its assets
#                    to check they are byte-identical to what was verified.
#   --collect DIR    Add the binary built on another machine: its .work/dist
#                    from scripts/dev/build.sh release, copied here together with
#                    the .work/dist.build-info.json beside it. Repeatable. Must
#                    be the same version and commit.
#   --allow-partial  Allow a release that does not contain all four platforms.
#   --publish        Re-verify the existing draft and, after you type the tag
#                    name (or pass --yes), make it public.
#
# Refuses to continue if: the working tree is not clean, the tag v<version> does
# not exist and point at HEAD, the tag is not on the remote (--execute), the
# release already exists (--execute), any check fails, a platform is missing
# (unless --allow-partial), or any checksum differs.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=build.sh
source "${SCRIPT_DIR}/build.sh"

require_gh() {
  command -v gh >/dev/null 2>&1 || die "the GitHub CLI (gh) is required for this step"
  gh auth status >/dev/null 2>&1 || die "gh is not authenticated; run: gh auth login"
}

require_clean_tree() {
  [ -z "$(git -C "${REPO_ROOT}" status --porcelain)" ] \
    || die "the working tree has uncommitted changes; commit or stash them so the release is reproducible"
}

require_tag_at_head() {
  local tag="$1" tag_commit head_commit
  tag_commit="$(git -C "${REPO_ROOT}" rev-parse -q --verify "refs/tags/${tag}^{commit}" 2>/dev/null)" \
    || die "tag ${tag} does not exist; create it on the release commit: git tag -a ${tag} -m ${tag}"
  head_commit="$(git -C "${REPO_ROOT}" rev-parse HEAD)"
  [ "${tag_commit}" = "${head_commit}" ] \
    || die "tag ${tag} points at ${tag_commit}, but HEAD is ${head_commit}; check out the tagged commit"
}

require_remote_tag() {
  local tag="$1" remote_commit head_commit
  remote_commit="$(git -C "${REPO_ROOT}" ls-remote --tags origin "refs/tags/${tag}" "refs/tags/${tag}^{}" \
    | awk '{ if ($2 ~ /\^\{\}$/) deref = $1; else plain = $1 } END { print (deref != "" ? deref : plain) }')"
  [ -n "${remote_commit}" ] || die "tag ${tag} is not on the remote; push it first: git push origin ${tag}"
  head_commit="$(git -C "${REPO_ROOT}" rev-parse HEAD)"
  [ "${remote_commit}" = "${head_commit}" ] || die "the remote tag ${tag} is ${remote_commit}, not HEAD (${head_commit})"
}

require_no_release() {
  if gh release view "$1" >/dev/null 2>&1; then
    die "a release named $1 already exists; delete it first or bump the version"
  fi
}

# Copy the asset(s) built on another machine into DIST_DIR. SOURCE is that
# machine's staged release directory; its <SOURCE>.build-info.json (written by
# build.sh release, beside the directory) says what it built, which is checked
# against this release's version and commit and against the file's checksum.
collect_from() {
  local source_dir="${1%/}" version="$2" revision="$3"
  [ -d "${source_dir}" ] || die "--collect: ${source_dir} is not a directory"
  [ -f "${source_dir}.build-info.json" ] \
    || die "--collect: ${source_dir}.build-info.json is missing; copy it next to the directory (build.sh release writes it)"
  echo "==> collect ${source_dir}"
  "${PYTHON}" - "${source_dir}" "${DIST_DIR}" "${TOOL}" "${version}" "${revision}" <<'PY'
import hashlib, json, os, shutil, sys
src, dst, tool, version, revision = sys.argv[1:6]
info = json.load(open(src + ".build-info.json"))
if info["tool"] != tool:
    sys.exit(f"{src} was built for {info['tool']!r}, not {tool!r}")
if info["version"] != version:
    sys.exit(f"{src} is version {info['version']}, this release is {version}")
if info["revision"] != revision:
    sys.exit(f"{src} was built from {info['revision']}, this release is {revision}")
for a in info["assets"]:
    path = os.path.join(src, a["file"])
    if not os.path.isfile(path):
        sys.exit(f"{a['file']} is missing from {src}")
    if hashlib.sha256(open(path, "rb").read()).hexdigest() != a["sha256"]:
        sys.exit(f"{a['file']} does not match its recorded checksum; it changed after it was built")
    target = os.path.join(dst, a["file"])
    if os.path.exists(target):
        sys.exit(f"{a['file']} is already staged; each platform must come from exactly one machine")
    shutil.copy2(path, target)
    print(f"added {a['file']}")
PY
}

# Download the (draft) release's assets and verify them.
download_and_verify_release() {
  local tag="$1" mode="$2" tmp
  tmp="$(mktemp -d)"
  gh release download "${tag}" --dir "${tmp}" >/dev/null
  verify_dist "${tmp}" "${mode}"
  DOWNLOADED_DIR="${tmp}"
}

prepare_release() {
  local version tag revision mode="all" dir
  version="$(crate_version)"; tag="v${version}"
  revision="$(git -C "${REPO_ROOT}" rev-parse HEAD)"
  echo "shipping ${TOOL} ${version} as ${tag} (commit ${revision})"
  [ "${ALLOW_PARTIAL}" = 1 ] && mode="any"

  require_clean_tree
  require_tag_at_head "${tag}"
  if [ "${EXECUTE}" = 1 ]; then
    require_gh
    require_remote_tag "${tag}"
    require_no_release "${tag}"
  fi

  step_pins; step_fmt; step_clippy; step_test; step_release
  for dir in ${COLLECT_DIRS[@]+"${COLLECT_DIRS[@]}"}; do
    collect_from "${dir}" "${version}" "${revision}"
  done
  write_checksums_and_manifest "${DIST_DIR}" "${version}" "${revision}"
  echo "==> verify the release as it will be published"
  verify_dist "${DIST_DIR}" "${mode}"

  echo "release contents (${DIST_DIR}):"
  ls -1 "${DIST_DIR}" | sed 's/^/  /'

  if [ "${EXECUTE}" != 1 ]; then
    echo "DRY RUN: nothing was published. Re-run with --execute to create the DRAFT release ${tag}."
    return 0
  fi

  echo "==> creating the DRAFT release ${tag}"
  gh release create "${tag}" --draft --verify-tag --title "${tag}" --generate-notes "${DIST_DIR}"/*
  echo "==> checking the uploaded draft is identical to what was verified"
  download_and_verify_release "${tag}" "${mode}"
  diff -r "${DIST_DIR}" "${DOWNLOADED_DIR}" >/dev/null \
    || die "the uploaded draft differs from the local release; do NOT publish it"
  echo "draft ${tag} created and verified: $(gh release view "${tag}" --json url -q .url)"
  echo "to make it public: scripts/dev/ship.sh --publish"
}

publish_release() {
  local version tag mode="all" is_draft tag_commit answer
  version="$(crate_version)"; tag="v${version}"
  [ "${ALLOW_PARTIAL}" = 1 ] && mode="any"
  require_gh
  is_draft="$(gh release view "${tag}" --json isDraft -q .isDraft 2>/dev/null)" \
    || die "release ${tag} does not exist; create the draft first: scripts/dev/ship.sh --execute"
  [ "${is_draft}" = "true" ] || die "release ${tag} is already public; nothing to do"
  echo "==> re-verifying the draft ${tag}"
  download_and_verify_release "${tag}" "${mode}"
  tag_commit="$(git -C "${REPO_ROOT}" rev-parse -q --verify "refs/tags/${tag}^{commit}")" \
    || die "tag ${tag} does not exist locally; fetch tags"
  if [ "${HAS_MANIFEST}" = 1 ]; then
    "${PYTHON}" - "${DOWNLOADED_DIR}/toolchain-manifest.json" "${tag_commit}" <<'PY'
import json, sys
m = json.load(open(sys.argv[1]))
if m["revision"] != sys.argv[2]:
    sys.exit(f"the draft was built from {m['revision']}, but the tag is {sys.argv[2]}")
PY
  fi
  if [ "${ASSUME_YES}" != 1 ]; then
    read -r -p "Type the tag name (${tag}) to make this release PUBLIC: " answer || answer=""
    [ "${answer}" = "${tag}" ] || die "confirmation did not match; nothing was published"
  fi
  gh release edit "${tag}" --draft=false --latest
  echo "published ${tag}: $(gh release view "${tag}" --json url -q .url)"
}

main() {
  EXECUTE=0; ALLOW_PARTIAL=0; PUBLISH=0; ASSUME_YES=0; COLLECT_DIRS=()
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --execute) EXECUTE=1 ;;
      --allow-partial) ALLOW_PARTIAL=1 ;;
      --publish) PUBLISH=1 ;;
      --yes) ASSUME_YES=1 ;;
      --collect) [ "$#" -ge 2 ] || die "--collect needs a directory"; COLLECT_DIRS+=("$2"); shift ;;
      -h|--help) sed -n '2,29p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
      *) die "unknown option '$1' (see --help)" ;;
    esac
    shift
  done
  if [ "${PUBLISH}" = 1 ] && { [ "${EXECUTE}" = 1 ] || [ "${#COLLECT_DIRS[@]}" -gt 0 ]; }; then
    die "--publish cannot be combined with --execute or --collect"
  fi
  init_environment
  if [ "${PUBLISH}" = 1 ]; then publish_release; else prepare_release; fi
}

main "$@"
