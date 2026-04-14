#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

LLVM_PREFIX="${LLVM_SYS_181_PREFIX:-/usr/lib/llvm-18}"
INPUT_FILE="${1:-}"
DEFAULT_INPUT_FILE=".test_input_default.txt"

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "missing required command: $1" >&2
        exit 1
    fi
}

require_command cargo
require_command clang-18
require_command aarch64-linux-gnu-gcc
require_command qemu-aarch64

if [[ -f "$HOME/.cargo/env" ]]; then
    # Load Rust toolchain installed by rustup.
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
fi

export LLVM_SYS_181_PREFIX="$LLVM_PREFIX"

echo "[1/4] generate LLVM IR and AArch64 assembly"
cargo run --features llvm-backend

echo "[2/4] build runtime object"
aarch64-linux-gnu-gcc -c sylib.c -o sylib.o

echo "[3/4] link ARM64 executable"
aarch64-linux-gnu-gcc output_aarch64.s sylib.o -o sysy_arm64

if [[ -n "$INPUT_FILE" ]]; then
    if [[ ! -f "$INPUT_FILE" ]]; then
        echo "input file not found: $INPUT_FILE" >&2
        exit 1
    fi
else
    printf '1\n3\n' > "$DEFAULT_INPUT_FILE"
    INPUT_FILE="$DEFAULT_INPUT_FILE"
fi

echo "[4/4] run with qemu-aarch64"
echo "input file: $INPUT_FILE"
set +e
qemu-aarch64 -L /usr/aarch64-linux-gnu ./sysy_arm64 < "$INPUT_FILE"
status=$?
set -e

echo "exit code: $status"
exit 0