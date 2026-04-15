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

use super::{
    error::HandlerResult,
    model::{data_flow::DataFlow, messages::DataFlowStatusMessage},
};

#[cfg(test)]
use crate::core::db::tx::MockTransaction;

#[async_trait::async_trait]
#[cfg_attr(test, mockall::automock(type Transaction = MockTransaction;))]
pub trait DataFlowHandler: Send + Sync {
    type Transaction;

    async fn can_handle(&self, flow: &DataFlow) -> HandlerResult<bool>;

    async fn on_start(
        &self,
        tx: &mut Self::Transaction,
        flow: &DataFlow,
    ) -> HandlerResult<DataFlowStatusMessage>;

    async fn on_prepare(
        &self,
        tx: &mut Self::Transaction,
        flow: &DataFlow,
    ) -> HandlerResult<DataFlowStatusMessage>;

    async fn on_terminate(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> HandlerResult<()>;
    async fn on_started(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> HandlerResult<()>;

    async fn on_suspend(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> HandlerResult<()>;
}
