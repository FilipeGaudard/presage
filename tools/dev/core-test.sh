#!/usr/bin/env bash
# cargo test do core no WSL:  bash tools/dev/core-test.sh [filtro]
# - o target fica fora do OneDrive (gigabytes que ele sincronizaria);
# - o presage-core é recompilado sempre, porque o mtime de /mnt/c anda uns 6 s
#   atrás do relógio do WSL e o cargo pode rodar o binário velho (L-28).
. "$HOME/.cargo/env"
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$HOME/.cache/presage-target}
cd "$(dirname "$0")/../.." || exit 9
cargo clean -q -p presage-core 2>/dev/null
exec cargo test -q -p presage-core "$@"
