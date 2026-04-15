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

use std::sync::Arc;

use dataplane_sdk::core::{
    db::tx::TransactionalContext,
    error::{HandlerError, HandlerResult},
    handler::DataFlowHandler,
    model::{
        data_flow::{DataFlow, DataFlowState},
        messages::DataFlowStatusMessage,
    },
};

use crate::tokens::{manager::TokenManager, model::Token};

#[derive(Clone)]
pub struct TokenHandler<T: TransactionalContext>(Arc<TokenManager<T>>);

impl<T: TransactionalContext> TokenHandler<T> {
    pub fn new(token_manager: Arc<TokenManager<T>>) -> Self {
        Self(token_manager)
    }
}

#[async_trait::async_trait]
impl<T: TransactionalContext> DataFlowHandler for TokenHandler<T>
where
    T::Transaction: Send,
{
    type Transaction = T::Transaction;

    async fn can_handle(&self, _flow: &DataFlow) -> HandlerResult<bool> {
        Ok(true)
    }

    async fn on_start(
        &self,
        tx: &mut Self::Transaction,
        flow: &DataFlow,
    ) -> HandlerResult<DataFlowStatusMessage> {
        let (token_id, endpoint, data_address) = self
            .0
            .create_token()
            .await
            .map_err(|err| HandlerError::Generic(Box::new(err)))?;

        let token = Token::builder()
            .flow_id(flow.id.clone())
            .dataset_id(flow.dataset_id.clone())
            .token_id(token_id)
            .endpoint(endpoint)
            .build();

        self.0
            .repo()
            .create(tx, token)
            .await
            .map_err(|err| HandlerError::Generic(err.into()))?;

        Ok(DataFlowStatusMessage::builder()
            .data_address(data_address)
            .state(DataFlowState::Started)
            .build())
    }
    async fn on_terminate(
        &self,
        _tx: &mut Self::Transaction,
        _flow: &DataFlow,
    ) -> HandlerResult<()> {
        Ok(())
    }

    async fn on_started(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> HandlerResult<()> {
        if let Some(data_address) = flow.data_address.as_ref() {
            let endpoint = data_address.get_property("endpoint").ok_or_else(|| {
                HandlerError::Generic("Data address must contain an endpoint property".into())
            })?;

            let token_id = data_address.get_property("access_token").ok_or_else(|| {
                HandlerError::Generic("Data address must contain an access_token property".into())
            })?;

            let token = Token::builder()
                .flow_id(flow.id.clone())
                .dataset_id(flow.dataset_id.clone())
                .token_id(token_id)
                .endpoint(endpoint)
                .build();

            self.0
                .repo()
                .create(tx, token)
                .await
                .map_err(|err| HandlerError::Generic(err.into()))?;

            Ok(())
        } else {
            return Err(HandlerError::Generic(
                "Data address is required to create token".into(),
            ));
        }
    }

    async fn on_prepare(
        &self,
        _tx: &mut Self::Transaction,
        _flow: &DataFlow,
    ) -> HandlerResult<DataFlowStatusMessage> {
        Ok(DataFlowStatusMessage::builder()
            .state(DataFlowState::Prepared)
            .build())
    }

    async fn on_suspend(&self, _tx: &mut Self::Transaction, _flow: &DataFlow) -> HandlerResult<()> {
        Ok(())
    }
}
