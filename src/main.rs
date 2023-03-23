mod constant;
mod dump;

use std::fs;

use clap::Parser;
use color_eyre::Result;
use starknet::core::types::FieldElement;

use constant::DUMP_STATE;
use dump::{dump, StorageSlot};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long)]
    contract: String,

    #[arg(short, long)]
    from_block: u64,

    #[arg(short, long)]
    to_block: u64,

    #[arg(long)]
    no_ui: bool,

    #[arg(short, long)]
    #[arg(value_name = "PATH")]
    #[arg(help = "The output file path.")]
    #[arg(default_value = "./output/storage_slot.json")]
    output: String,

    #[arg(short = 'u', long)]
    #[arg(env = "STARKNET_RPC_URL")]
    rpc_url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cli {
        no_ui,
        output,
        rpc_url,
        contract,
        to_block,
        from_block,
    } = Cli::parse();

    if no_ui {
        dump(
            Box::leak(Box::new(rpc_url)),
            &DUMP_STATE,
            FieldElement::from_hex_be(&contract)?,
            from_block,
            to_block,
        )
        .await?;

        let state = DUMP_STATE.lock().unwrap();
        let storages = state
            .storage
            .iter()
            .map(|(key, value)| StorageSlot {
                key: key.clone(),
                value: value.clone(),
            })
            .collect::<Vec<_>>();

        let json = serde_json::to_string_pretty(&storages)?;
        fs::write(output, json)?;
    } else {
        todo!("dump with ui")
    }

    Ok(())
}
