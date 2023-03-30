# Jin

A tool for dumping storage slots of a Starknet contract.

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

## Note

_If you are using a RPC provider like INFURA, you might want to update the rate limits depending on the total of blocks you are querying because the requests will be done in parallel._
