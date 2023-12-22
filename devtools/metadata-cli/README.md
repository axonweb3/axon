# Metadata Cli

Cli for working with axon metadata cell.

## Get Data

Generate `MetadataCellData` from validators.

Usage:

From a chain spec file or input file:

```command
axon-metadata get-data -i input.example.toml
```

```command
axon-metadata get-data -i chain-spec.toml
```

From JSONRPC metadata result:

```command
curl 'https://rpc-alphanet-axon.ckbapp.dev/' --header 'Content-Type: application/json' -d '{"jsonrpc":"2.0","method":"axon_getCurrentMetadata","id":3}' | jq '.result' | axon-metadata get-data -i /dev/stdin
```

### Deploy the Metadata Cell

After generating the cell data, you can deploy a metadata type-id cell with e.g. `ckb-cli wallet transfer` or `ckb-cli deploy`.

## Parse Data

Parse `MetadataCellData` to get validators.

Usage:

```command
axon-metadata parse-data -i data.example.hex
```

From JSONRPC cell data:

```command
curl 'https://testnet.ckbapp.dev/' --header 'Content-Type: application/json' -d '{"jsonrpc":"2.0","method":"get_live_cell","params":[{"index":"0x0","tx_hash":"0x8a37967294c40da9ede155156bfe87d4b4e644c2b7f3275dd2ec4ebe4d695e24"},true],"id":3}' | jq -r '.result.cell.data.content' | axon-metadata parse-data -i /dev/stdin
```
