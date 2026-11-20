#!/usr/bin/env bash
# Checks the inputs for the chosen mode and runs atsgate. Inputs arrive as environment variables,
# never interpolated into the script, so no input can inject shell.
set -euo pipefail

fail() {
  echo "::error::$1"
  exit 1
}

need() {
  local name="$1" value="$2"
  [[ -n "$value" ]] || fail "mode $INPUT_MODE needs the input '$name'"
}

build_args=()
add_build() {
  need career "$INPUT_CAREER"
  need tailoring "$INPUT_TAILORING"
  need contact "$INPUT_CONTACT"
  need out "$INPUT_OUT"
  build_args+=(--career "$INPUT_CAREER" --tailoring "$INPUT_TAILORING"
    --contact "$INPUT_CONTACT" --out "$INPUT_OUT")
  case "$INPUT_ALLOW_PRIVATE_CONTACT" in
    true) build_args+=(--allow-private-contact) ;;
    false) ;;
    *) fail "allow-private-contact must be true or false, not '$INPUT_ALLOW_PRIVATE_CONTACT'" ;;
  esac
}

score_args=()
add_score() {
  need blueprint "$INPUT_BLUEPRINT"
  need fail-on-integrity-flags "$INPUT_FAIL_ON_INTEGRITY_FLAGS"
  need report "$INPUT_REPORT"
  case "$INPUT_FAIL_ON_INTEGRITY_FLAGS" in
    true | false) ;;
    *) fail "fail-on-integrity-flags must be true or false, not '$INPUT_FAIL_ON_INTEGRITY_FLAGS'" ;;
  esac
  [[ "$INPUT_REPORT" == *.json ]] || fail "report must end in .json, not '$INPUT_REPORT'"
  score_args+=(--blueprint "$INPUT_BLUEPRINT"
    --fail-on-integrity-flags "$INPUT_FAIL_ON_INTEGRITY_FLAGS"
    --as-of today --report "$INPUT_REPORT" --summary "${INPUT_REPORT%.json}.md")
  if [[ -n "$INPUT_FLOORS" ]]; then
    score_args+=(--floors "$INPUT_FLOORS")
  fi
}

case "$INPUT_MODE" in
  build)
    add_build
    exec "$ATSGATE_BIN" build "${build_args[@]}"
    ;;
  score)
    need resume "$INPUT_RESUME"
    add_score
    exec "$ATSGATE_BIN" score --resume "$INPUT_RESUME" "${score_args[@]}"
    ;;
  gate)
    add_build
    add_score
    exec "$ATSGATE_BIN" gate "${build_args[@]}" "${score_args[@]}"
    ;;
  *) fail "mode must be build, score or gate, not '$INPUT_MODE'" ;;
esac
