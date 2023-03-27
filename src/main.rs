mod constant;
mod dump;
mod utils;

use clap::Parser;
use color_eyre::Result;
use starknet::core::types::FieldElement;

use constant::DUMP_STATE;
use dump::{dump, StorageSlot};
use utils::dump_to_file;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(short, long)]
    contract: String,

    #[arg(short, long)]
    from_block: u64,

    #[arg(short, long)]
    to_block: u64,

    #[arg(long)]
    ui: bool,

    #[arg(short, long)]
    #[arg(value_name = "PATH")]
    #[arg(help = "The output file path.")]
    output: Option<String>,

    #[arg(short = 'u', long)]
    #[arg(env = "STARKNET_RPC_URL")]
    rpc_url: String,
}

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

    if ui {
        todo!("dump with ui")
    } else {
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
                key: *key,
                value: value.clone(),
            })
            .collect::<Vec<_>>();

        dump_to_file(output, storages)?;
    }

    Ok(())
}
