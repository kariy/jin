use color_eyre::Result;
use indicatif::ProgressBar;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::{
    serde::unsigned_field_element::UfeHex,
    types::{BlockId, FieldElement},
};
use starknet::providers::{Provider, SequencerGatewayProvider};

use std::{collections::BTreeMap, sync::Mutex};

#[serde_as]
#[derive(Debug, Serialize)]
pub struct StorageSlot {
    #[serde_as(as = "UfeHex")]
    pub key: FieldElement,
    pub value: StorageValue,
}

#[serde_as]
#[derive(Debug, Serialize, Clone)]
pub struct StorageValue {
    #[serde_as(as = "UfeHex")]
    pub value: FieldElement,
    pub last_updated_block: u64,
}

pub struct DumpState {
    pub storage: BTreeMap<FieldElement, StorageValue>,
}

impl DumpState {
    pub fn new() -> Self {
        Self {
            storage: BTreeMap::new(),
        }
    }
}

// TODO: parallelize this somehow
pub async fn fetch_contract_storage(
    dump_state: &Mutex<DumpState>,
    contract: &FieldElement,
    from_block: u64,
    to_block: u64,
) -> Result<()> {
    let client = SequencerGatewayProvider::starknet_alpha_mainnet();

    let progress_bar = ProgressBar::new(to_block - from_block);

    for i in from_block..=to_block {
        let res = client.get_state_update(BlockId::Number(i)).await?;

        let found = res
            .state_diff
            .storage_diffs
            .iter()
            .find(|c| c.0 == contract);

        if let Some((_, diffs)) = found {
            diffs.iter().for_each(|d| {
                let mut state = dump_state.lock().unwrap();
                state.storage.insert(
                    d.key,
                    StorageValue {
                        value: d.value,
                        last_updated_block: i,
                    },
                );
            });
        }

        progress_bar.inc(1);
    }

    progress_bar.finish_and_clear();

    Ok(())
}
