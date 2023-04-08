# jin

A tool for dumping storage slots of a Starknet contract.

## How it works

This tool relies on the `starknet_getStateUpdate` endpoint. It will query the state updates of each block from `--from-block` to `--to-block` and filter the storage changes of only the specified `--contract`.

In order to ensure you fetch the full contract state properly, you would have to set `--from-block` to the block number the contract was deployed at and set `--to-block` to the latest block.

## Usage

```
Usage: jin [OPTIONS] --contract <CONTRACT> --from-block <FROM_BLOCK> --to-block <TO_BLOCK> --rpc-url <RPC_URL>

Options:
  -c, --contract <CONTRACT>      The contract address whose storage to be dumped.
  -f, --from-block <FROM_BLOCK>
  -t, --to-block <TO_BLOCK>
      --ui                       Display a UI for browsing through the contract storages.
  -o, --output <PATH>            The output file path.
  -u, --rpc-url <RPC_URL>        The RPC endpoint. [env: STARKNET_RPC_URL=]
  -h, --help                     Print help
```

The storage data will be saved in the `--output` file `(default: ./output/storage_slot.json)`.

## Keybinds

```
<UP> Select previous item.
<DOWN> Select next item.
<ESC> Exit the UI.
```

## Note

_If you are using a RPC provider like INFURA, you might have to update the rate limits depending on the total of blocks you are querying because the requests will be done in parallel._
