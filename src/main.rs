mod constant;
mod dump;
mod ui;
mod utils;

use clap::Parser;
use color_eyre::Result;
use starknet::core::types::FieldElement;
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet::providers::Provider;
use url::Url;

use constant::DUMP_STATE;
use dump::{dump, StorageSlot};
use ui::execute_ui;
use utils::dump_to_file;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short = 'C', long)]
    #[arg(help = "The contract address whose storage to be dumped.")]
    contract: FieldElement,

    #[arg(short, long)]
    #[arg(value_name = "BLOCK_NUMBER")]
    from_block: u64,

    #[arg(short, long)]
    #[arg(value_name = "BLOCK_NUMBER")]
    #[arg(help = "By default, it will fetch until the latest block.")]
    to_block: Option<u64>,

    #[arg(long)]
    #[arg(help = "Display a UI for browsing through the contract storages.")]
    ui: bool,

    #[arg(short, long)]
    #[arg(value_name = "PATH")]
    #[arg(help = "The output file path. Default: ./output/storage_slot.json")]
    output: Option<String>,

    #[arg(short = 'u', long)]
    #[arg(env = "STARKNET_RPC_URL")]
    #[arg(help = "The RPC endpoint.")]
    rpc_url: String,
}

pub struct Config {
    to_block: u64,
    from_block: u64,
    contract: FieldElement,
}

// TODO: structure it better
#[tokio::main]
async fn main() -> Result<()> {
    let Cli {
        ui,
        output,
        rpc_url,
        contract,
        to_block,
        from_block,
    } = Cli::parse();

    let to_block = if let Some(block) = to_block {
        block
    } else {
        fetch_latest_block(&rpc_url).await?
    };

    let config = Config {
        to_block,
        from_block,
        contract,
    };

    dump(
        Box::leak(Box::new(rpc_url)),
        &DUMP_STATE,
        contract,
        from_block,
        to_block,
    )
    .await?;

    let state = DUMP_STATE.lock().unwrap();
    let storages = state
        .storage
        .iter()
        .map(|(key, value)| StorageSlot {
            key: *key,
            value: value.clone(),
        })
        .collect::<Vec<_>>();

    if ui {
        std::thread::spawn(|| {
            dump_to_file(output, storages).expect("unable to write to output file");
        });

        execute_ui(&state, config)?;
    } else {
        dump_to_file(output, storages)?;
    }

    Ok(())
}

async fn fetch_latest_block(rpc_url: impl AsRef<str>) -> Result<u64> {
    let client = JsonRpcClient::new(HttpTransport::new(Url::parse(rpc_url.as_ref()).unwrap()));
    client.block_number().await.map_err(|e| e.into())
}
