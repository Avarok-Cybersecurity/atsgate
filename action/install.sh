#!/usr/bin/env bash
# Puts an atsgate binary on disk and writes its path to the step output `bin`.
#
# - `uses: ./` (ACTION_REPOSITORY empty): builds this checkout from source, so the action's own
#   CI tests the code in the pull request.
# - Anywhere else: downloads the release binary for this runner named in release-pins.txt and
#   refuses it unless its sha256 matches the pin committed beside this script.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(dirname "$here")"
repo="Avarok-Cybersecurity/atsgate"

if [[ -z "${ACTION_REPOSITORY}" ]]; then
  cd "$root"
  rustup toolchain install
  cargo build --release --locked -p atsgate-cli
  exe="$root/target/release/atsgate"
  [[ "$RUNNER_OS_NAME" == "Windows" ]] && exe="$exe.exe"
  echo "bin=$exe" >> "$GITHUB_OUTPUT"
  exit 0
fi

case "$RUNNER_OS_NAME/$RUNNER_ARCH_NAME" in
  Linux/X64) asset="atsgate-x86_64-unknown-linux-gnu" ;;
  Linux/ARM64) asset="atsgate-aarch64-unknown-linux-gnu" ;;
  macOS/ARM64) asset="atsgate-aarch64-apple-darwin" ;;
  Windows/X64) asset="atsgate-x86_64-pc-windows-msvc.exe" ;;
  *)
    echo "::error::atsgate has no release binary for $RUNNER_OS_NAME/$RUNNER_ARCH_NAME"
    exit 1
    ;;
esac

# Lines of "<version> <asset> <sha256>"; comments and blank lines are ignored.
pins="$here/release-pins.txt"
pin="$(grep -v '^[[:space:]]*\(#\|$\)' "$pins" | awk -v a="$asset" '$2 == a' || true)"
if [[ -z "$pin" ]]; then
  echo "::error::this revision of the action pins no release binary for $asset; use a release tag of $repo"
  exit 1
fi
if [[ "$(wc -l <<<"$pin")" -ne 1 ]]; then
  echo "::error::release-pins.txt pins $asset more than once"
  exit 1
fi
read -r version _ want <<<"$pin"

dir="$RUNNER_TEMP/atsgate-$version"
mkdir -p "$dir"
exe="$dir/$asset"
curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
  --retry 3 --output "$exe" \
  "https://github.com/$repo/releases/download/$version/$asset"
if command -v sha256sum >/dev/null; then
  got="$(sha256sum "$exe" | awk '{print $1}')"
else
  got="$(shasum -a 256 "$exe" | awk '{print $1}')"
fi
if [[ "$got" != "$want" ]]; then
  echo "::error::$asset $version has sha256 $got; the action pins $want"
  rm -f "$exe"
  exit 1
fi
chmod +x "$exe"
echo "bin=$exe" >> "$GITHUB_OUTPUT"
