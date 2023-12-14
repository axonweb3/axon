# Metadata Cli

Cli for working with axon metadata cell.

## Get Data

Generate `MetadataCellData` from validators.

Usage:

From a chain spec file or input file:

```command
$ axon-metadata get-data -i input.example.toml
```

```command
$ axon-metadata get-data -i chain-spec.toml
```

From JSONRPC metadata result:

```command
$ curl 'https://rpc-alphanet-axon.ckbapp.dev/' --header 'Content-Type: application/json' -d '{"jsonrpc":"2.0","method":"axon_getCurrentMetadata","id":3}' | jq '.result' | axon-metadata get-data -i /dev/stdin
```
