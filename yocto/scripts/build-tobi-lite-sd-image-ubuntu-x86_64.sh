#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

MACHINE="${MACHINE:-am62xxsip-evm}" \
BITBAKE_TARGET="${BITBAKE_TARGET:-tobi-lite-sd-image}" \
exec "$SCRIPT_DIR/build-tobi-sd-image-ubuntu-x86_64.sh"
