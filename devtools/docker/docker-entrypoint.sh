#!/bin/sh

DATA_DIR="/app/devtools/chain/data"
CONFIG_FILE="/app/devtools/chain/config.toml"
CHAIN_SPEC_FILE="/app/devtools/chain/specs/single_node/chain-spec.toml"

if [ ! -e "${DATA_DIR}" ]; then
    /app/axon init -c=${CONFIG_FILE} -s=${CHAIN_SPEC_FILE}
fi
/app/axon run -c=${CONFIG_FILE}
