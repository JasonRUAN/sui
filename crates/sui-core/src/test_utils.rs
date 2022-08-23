// Copyright (c) 2022, Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use crate::{
    authority::AuthorityState,
    authority_aggregator::{AuthAggMetrics, AuthorityAggregator},
    authority_client::NetworkAuthorityClient,
    epoch::epoch_store::EpochStore,
    safe_client::SafeClientMetrics,
};

use sui_config::{NetworkConfig, ValidatorInfo};
use sui_types::{
    base_types::TransactionDigest,
    batch::UpdateItem,
    committee::Committee,
    messages::{BatchInfoRequest, BatchInfoResponseItem},
    object::Object,
};

// Can't import SuiNode directly from sui_node - circular dependency
use test_utils::authority::{start_node, SuiNode};

use futures::StreamExt;
use tokio::time::sleep;
use tracing::info;

/// Spawn all authorities in the test committee into a separate tokio task.
pub async fn spawn_test_authorities<I>(objects: I, config: &NetworkConfig) -> Vec<SuiNode>
where
    I: IntoIterator<Item = Object> + Clone,
{
    let mut handles = Vec::new();
    for validator in config.validator_configs() {
        let node = start_node(validator).await;
        let state = node.state();

        for o in objects.clone() {
            state.insert_genesis_object(o).await
        }

        handles.push(node);
    }
    handles
}

/// Create a test authority aggregator.
/// (duplicated from test-utils/src/authority.rs - that function can't be used
/// in sui-core because of type name conflicts (sui_core::safe_client::SafeClient vs
/// safe_client::SafeClient).
pub fn test_authority_aggregator(
    config: &NetworkConfig,
) -> AuthorityAggregator<NetworkAuthorityClient> {
    let validators_info = config.validator_set();
    let committee = Committee::new(0, ValidatorInfo::voting_rights(validators_info)).unwrap();
    let epoch_store = Arc::new(EpochStore::new_for_testing(&committee));
    let clients: BTreeMap<_, _> = validators_info
        .iter()
        .map(|config| {
            (
                config.public_key(),
                NetworkAuthorityClient::connect_lazy(config.network_address()).unwrap(),
            )
        })
        .collect();
    let registry = prometheus::Registry::new();
    AuthorityAggregator::new(
        committee,
        epoch_store,
        clients,
        AuthAggMetrics::new(&registry),
        SafeClientMetrics::new(&registry),
    )
}

pub async fn wait_for_tx(wait_digest: TransactionDigest, state: Arc<AuthorityState>) {
    wait_for_all_txes(vec![wait_digest], state).await
}

pub async fn wait_for_all_txes(wait_digests: Vec<TransactionDigest>, state: Arc<AuthorityState>) {
    let mut wait_digests: HashSet<_> = wait_digests.iter().collect();

    let mut timeout = Box::pin(sleep(Duration::from_millis(15_000)));

    let mut max_seq = Some(0);

    let mut stream = Box::pin(
        state
            .handle_batch_streaming(BatchInfoRequest {
                start: max_seq,
                length: 1000,
            })
            .await
            .unwrap(),
    );

    loop {
        tokio::select! {
            _ = &mut timeout => panic!("wait_for_tx timed out"),

            items = &mut stream.next() => {
                match items {
                    // Upon receiving a batch
                    Some(Ok(BatchInfoResponseItem(UpdateItem::Batch(batch)) )) => {
                        max_seq = Some(batch.data().next_sequence_number);
                        info!(?max_seq, "Received Batch");
                    }
                    // Upon receiving a transaction digest we store it, if it is not processed already.
                    Some(Ok(BatchInfoResponseItem(UpdateItem::Transaction((_seq, digest))))) => {
                        info!(?digest, "Received Transaction");
                        if wait_digests.remove(&digest.transaction) {
                            info!(?digest, "Digest found");
                        }
                        if wait_digests.is_empty() {
                            info!(?digest, "all digests found");
                            break;
                        }
                    },

                    Some(Err( err )) => panic!("{}", err),
                    None => {
                        info!(?max_seq, "Restarting Batch");
                        stream = Box::pin(
                                state
                                    .handle_batch_streaming(BatchInfoRequest {
                                        start: max_seq,
                                        length: 1000,
                                    })
                                    .await
                                    .unwrap(),
                            );

                    }
                }
            },
        }
    }
}
