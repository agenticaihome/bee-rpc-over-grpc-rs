#!/usr/bin/env bash
# The files of the specification and their SHA-256. The implementation agent must
# not change them (see AGENT_GUIDE.md, section 2.1).
#
#   tools/spec-hash.sh           print the list
#   tools/spec-hash.sh --check   compare it with SPEC.sha256; exit 1 on a difference
#   tools/spec-hash.sh --write   write SPEC.sha256 (maintainers only)
set -euo pipefail
cd "$(dirname "$0")/.."

list() {
  find crates/bee-rpc/tests crates/bee-rpc/protos crates/bee-rpc/src/gen \
       tools/vectors tools/interop -type f ! -path '*/__pycache__/*' -print0 \
    | LC_ALL=C sort -z | xargs -0 sha256sum
  sha256sum DEVIATIONS.md AGENT_GUIDE.md PROMPT.md tools/spec-hash.sh
}

case "${1:-}" in
  --check)
    if ! diff <(list) SPEC.sha256; then
      echo "The spec files differ from SPEC.sha256 (lines above: < now, > expected)." >&2
      exit 1
    fi ;;
  --write) list > SPEC.sha256 ;;
  *) list ;;
esac
