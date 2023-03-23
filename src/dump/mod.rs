use color_eyre::Result;
use indicatif::ProgressBar;
use parallel_stream::prelude::*;
use rayon::prelude::*;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::{serde::unsigned_field_element::UfeHex, types::FieldElement};
use starknet::providers::jsonrpc::models::BlockId;
use starknet::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use url::Url;

use std::{collections::BTreeMap, sync::Mutex, time::Duration};

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

// TODO: increment progress bar in every stream
pub async fn dump(
    url: &'static str,
    dump_state: &'static Mutex<DumpState>,
    contract: FieldElement,
    from_block: u64,
    to_block: u64,
) -> Result<()> {
    let progress_bar = ProgressBar::new_spinner();
    progress_bar.enable_steady_tick(Duration::from_millis(150));
    // progress_bar.inc(1);

    (from_block..=to_block)
        .collect::<Vec<u64>>()
        .into_par_stream()
        .for_each(move |i| async move {
            let client = JsonRpcClient::new(HttpTransport::new(Url::parse(url).unwrap()));
            let res = client.get_state_update(&BlockId::Number(i)).await;

            if let Err(err) = &res {
                println!("Got err {}", err);
                return;
            };

            let state_update = res.unwrap();
            let found = state_update
                .state_diff
                .storage_diffs
                .par_iter()
                .find_any(|c| c.address == contract);

            if let Some(storage_diff) = found {
                storage_diff.storage_entries.par_iter().for_each(|d| {
                    let mut state = dump_state.lock().unwrap();
                    let exist_slot = state.storage.get(&d.key);

                    if let Some(slot) = exist_slot {
                        if i <= slot.last_updated_block {
                            return;
                        }
                    }

                    state.storage.insert(
                        d.key,
                        StorageValue {
                            value: d.value,
                            last_updated_block: i,
                        },
                    );
                });
            }
        })
        .await;

    progress_bar.finish_and_clear();

    Ok(())
}
