#!/bin/sh

BASE_DIR="/app/devtools/chain"
DATA_DIR="${BASE_DIR}/data"
CONFIG_FILE="${BASE_DIR}/config.toml"
CHAIN_SPEC_FILE="${BASE_DIR}/specs/single_node/chain-spec.toml"
KEY_FILE="${BASE_DIR}/debug.key"

if [ ! -e "${DATA_DIR}" ]; then
    /app/axon init --config "${CONFIG_FILE}" --chain-spec "${CHAIN_SPEC_FILE}" --key-file "${KEY_FILE}"
fi
/app/axon run --config "${CONFIG_FILE}"
