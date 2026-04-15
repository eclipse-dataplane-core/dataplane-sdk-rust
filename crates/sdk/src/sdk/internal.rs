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

use crate::{
    core::{
        db::{
            data_flow::DataFlowRepo,
            tx::{Transaction, TransactionalContext},
        },
        error::{DbError, HandlerError},
        handler::DataFlowHandler,
        model::{
            data_flow::{DataFlow, DataFlowState, DataFlowType},
            messages::{
                DataFlowPrepareMessage, DataFlowStartMessage, DataFlowStartedNotificationMessage,
                DataFlowStatusMessage,
            },
        },
    },
    error::{SdkError, SdkResult},
};

pub struct DataPlaneSdkInternal<C>
where
    C: TransactionalContext,
{
    pub(crate) ctx: C,
    pub(crate) repo: Box<dyn DataFlowRepo<Transaction = C::Transaction>>,
    pub(crate) handler: Box<dyn DataFlowHandler<Transaction = C::Transaction>>,
}

impl<C> DataPlaneSdkInternal<C>
where
    C: TransactionalContext,
{
    pub async fn start(
        &self,
        participant_context_id: &str,
        req: DataFlowStartMessage,
    ) -> SdkResult<DataFlowStatusMessage> {
        let flow = DataFlow::builder()
            .id(req.process_id)
            .counter_party_id(req.counter_party_id)
            .maybe_data_address(req.data_address)
            .participant_context_id(participant_context_id)
            .state(DataFlowState::Initiating)
            .metadata(req.metadata)
            .participant_id(req.participant_id)
            .dataspace_context(req.dataspace_context)
            .dataset_id(req.dataset_id)
            .agreement_id(req.agreement_id)
            .callback_address(req.callback_address)
            .labels(req.labels)
            .transfer_type(req.transfer_type)
            .kind(DataFlowType::Provider)
            .build();

        if self.handler.can_handle(&flow).await? {
            let mut tx = self.ctx.begin().await?;
            self.repo.create(&mut tx, &flow).await?;
            let response = self.handler.on_start(&mut tx, &flow).await?;
            tx.commit().await?;

            Ok(response)
        } else {
            Err(SdkError::Handler(HandlerError::NotSupported(
                "Data flow handler cannot handle this flow".to_string(),
            )))
        }
    }

    pub async fn prepare(
        &self,
        participant_context_id: &str,
        req: DataFlowPrepareMessage,
    ) -> SdkResult<DataFlowStatusMessage> {
        let mut flow = DataFlow::builder()
            .id(req.process_id)
            .counter_party_id(req.counter_party_id)
            .participant_context_id(participant_context_id)
            .state(DataFlowState::Initiating)
            .metadata(req.metadata)
            .participant_id(req.participant_id)
            .dataspace_context(req.dataspace_context)
            .dataset_id(req.dataset_id)
            .agreement_id(req.agreement_id)
            .callback_address(req.callback_address)
            .labels(req.labels)
            .transfer_type(req.transfer_type)
            .kind(DataFlowType::Consumer)
            .build();

        if self.handler.can_handle(&flow).await? {
            let mut tx = self.ctx.begin().await?;
            let response = self.handler.on_prepare(&mut tx, &flow).await?;
            flow.transition_to_prepared()?;
            self.repo.create(&mut tx, &flow).await?;
            tx.commit().await?;

            Ok(response)
        } else {
            Err(SdkError::Handler(HandlerError::NotSupported(
                "Data flow handler cannot handle this flow".to_string(),
            )))
        }
    }

    pub async fn terminate(
        &self,
        _ctx: &str,
        flow_id: &str,
        reason: Option<String>,
    ) -> SdkResult<()> {
        let mut tx = self.ctx.begin().await?;
        let mut flow = self
            .repo
            .fetch_by_id(&mut tx, flow_id)
            .await?
            .ok_or_else(|| DbError::NotFound(flow_id.to_string()))?;

        flow.transition_to_terminated(reason)?;
        self.repo.update(&mut tx, &flow).await?;

        self.handler.on_terminate(&mut tx, &flow).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn started(
        &self,
        _ctx: &str,
        flow_id: &str,
        msg: DataFlowStartedNotificationMessage,
    ) -> SdkResult<()> {
        let mut tx = self.ctx.begin().await?;
        let mut flow = self
            .repo
            .fetch_by_id(&mut tx, flow_id)
            .await?
            .ok_or_else(|| DbError::NotFound(flow_id.to_string()))?;

        flow.data_address = msg.data_address;

        self.handler.on_started(&mut tx, &flow).await?;

        flow.transition_to_started()?;
        self.repo.update(&mut tx, &flow).await?;

        tx.commit().await?;
        Ok(())
    }

    pub async fn suspend(
        &self,
        _ctx: &str,
        flow_id: &str,
        reason: Option<String>,
    ) -> SdkResult<()> {
        let mut tx = self.ctx.begin().await?;

        let mut flow = self
            .repo
            .fetch_by_id(&mut tx, flow_id)
            .await?
            .ok_or_else(|| DbError::NotFound(flow_id.to_string()))?;

        flow.transition_to_suspended(reason)?;
        self.repo.update(&mut tx, &flow).await?;

        self.handler.on_suspend(&mut tx, &flow).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn fetch_by_id(
        &self,
        tx: &mut C::Transaction,
        flow_id: &str,
    ) -> SdkResult<Option<DataFlow>> {
        self.repo.fetch_by_id(tx, flow_id).await.map(Ok)?
    }

    pub fn ctx(&self) -> &C {
        &self.ctx
    }
}
