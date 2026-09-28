//  Copyright (c) 2026 Metaform Systems, Inc
//
//  This program and the accompanying materials are made available under the
//  terms of the Apache License, Version 2.0 which is available at
//  https://www.apache.org/licenses/LICENSE-2.0
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Contributors:
//         Metaform Systems, Inc. - initial API and implementation
//

use crate::core::{error::DbResult, model::data_flow::DataFlow};
pub mod memory;

#[cfg(test)]
use crate::core::db::tx::MockTransaction;

#[async_trait::async_trait]
#[cfg_attr(test, mockall::automock(type Transaction = MockTransaction;))]
pub trait DataFlowRepo: Send + Sync {
    type Transaction;

    /// Creates a flow. Flow ids are unique per participant context, not globally: two participant
    /// contexts may each own a flow with the same id.
    async fn create(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> DbResult<()>;

    /// Fetches the flow `flow_id` owned by `participant_context_id`. A flow owned by another
    /// participant context is reported as absent (`Ok(None)`), never returned.
    async fn fetch_by_id(
        &self,
        tx: &mut Self::Transaction,
        participant_context_id: &str,
        flow_id: &str,
    ) -> DbResult<Option<DataFlow>>;

    /// Updates the flow identified by (`flow.participant_context_id`, `flow.id`).
    async fn update(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> DbResult<()>;

    /// Deletes the flow `flow_id` owned by `participant_context_id`.
    async fn delete(
        &self,
        tx: &mut Self::Transaction,
        participant_context_id: &str,
        flow_id: &str,
    ) -> DbResult<()>;
}
