#!/bin/bash
block_number() {
    block=$(curl -s 'http://127.0.0.1:8000' \
            --header 'Content-Type: application/json' \
            --data '{"jsonrpc":"2.0", "method":"eth_blockNumber", "params": [], "id":42}' | jq '.result' |xargs printf %d 0xF)
    echo $block
}




block_stats() {
    current_block=$(block_number)
    start_time=$(date +%s)
    wait_seconds=60
    
    while true; do
        latest_block=$(block_number)
        if [ $current_block -lt  $latest_block ]
        then
            return 0
        fi

        current_time=$(date +%s)
        elapsed_seconds=$((current_time - start_time))
        if [ $elapsed_seconds -ge $wait_seconds ]
        then
                break
        fi

        sleep 1
    done

    echo "block does not grow in one minute, please check"
    return 1
}


block_stats