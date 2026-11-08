#!/usr/bin/env bash
if [[ -d /home/ruv/.local/share/dream-machine/runtime/node-v24.21.0-linux-x64/bin ]]; then
  export PATH="/home/ruv/.local/share/dream-machine/runtime/node-v24.21.0-linux-x64/bin:/home/ruv/.cargo/bin:${PATH}"
fi
export PLAYWRIGHT_BROWSERS_PATH="${PLAYWRIGHT_BROWSERS_PATH:-/home/ruv/.cache/ms-playwright}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/canopy-relay-target}"
export CR_BIND="${CR_BIND:-127.0.0.1:18080}"
export CR_BASE_URL="${CR_BASE_URL:-http://${CR_BIND}}"
export CR_FRONTEND_URL="${CR_FRONTEND_URL:-http://127.0.0.1:5173}"
export CR_TOKEN="${CR_TOKEN:-canopy-relay-field-lead-token}"
export CR_SERVICE_PACKAGE="${CR_SERVICE_PACKAGE:-cr-service}"
export CR_RUN_DIR="${DREAM_MACHINE_RUN_DIR:-${CR_RUN_DIR:-/tmp/canopy-relay-runtime}}"
export CR_DATA="${CR_DATA:-${CR_RUN_DIR}/data}"
export CR_PID_DIR="${CR_PID_DIR:-${CR_RUN_DIR}/pids}"
export CANOPY_RELAY_ADDR="${CANOPY_RELAY_ADDR:-$CR_BIND}"
export CANOPY_RELAY_DATA="${CANOPY_RELAY_DATA:-$CR_DATA}"
