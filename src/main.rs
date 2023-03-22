mod constant;
mod dump;

use std::fs;

use clap::{ArgAction, Parser};
use color_eyre::Result;
use starknet::core::types::FieldElement;

use constant::DUMP_STATE;
use dump::{fetch_contract_storage, StorageSlot};

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long)]
    contract: String,

    #[arg(long)]
    from_block: u64,

    #[arg(long)]
    to_block: u64,

    #[arg(long)]
    #[arg(action = ArgAction::SetFalse)]
    no_ui: bool,

    #[arg(short, long)]
    #[arg(value_name = "PATH")]
    #[arg(help = "The output file path.")]
    #[arg(default_value = "./output/storage_slot.json")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let Cli {
        no_ui,
        output,
        contract,
        to_block,
        from_block,
    } = Cli::parse();

    fetch_contract_storage(
        &DUMP_STATE,
        &FieldElement::from_hex_be(&contract)?,
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

    Ok(())
}
