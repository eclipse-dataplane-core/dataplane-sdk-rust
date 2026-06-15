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

use std::collections::HashMap;

use dataplane_sdk::core::{
    error::{HandlerError, HandlerResult},
    handler::DataFlowHandler,
    model::{
        data_address::DataAddress,
        data_flow::{DataFlow, DataFlowState, DataFlowType},
        messages::DataFlowStatusMessage,
    },
};
use dataplane_sdk_postgres::PgTransaction;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod util;

#[cfg(test)]
mod tck_tests {
    use crate::util::TckTestReporter;

    use super::*;

    static EXPECTED_FAILURES: &[&str] = &[
        "DP_P_PULL:04-01",
        "DP_C_PULL:03-01",
        "DP_C_PULL:04-01",
        "DP_C_PUSH:04-01",
        "DP_P_PUSH:03-01",
        "DP_P_PUSH:04-01",
    ];

    #[tokio::test]
    async fn dataplane_tck_test() {
        tracing_subscriber::registry()
            .with(env_filter())
            .with(tracing_subscriber::fmt::layer())
            .init();
        let (ctx, repo, _container) = util::setup_postgres_container().await;
        let sdk = util::sdk(ctx, repo, TckTestHandler::new()).await;

        util::start_signaling(8282, sdk).await;

        let reporter = TckTestReporter::default();

        let _tck_container = util::setup_tck_container(reporter.clone()).await;

        let mut failures = reporter.failures();
        failures.retain(|f| !EXPECTED_FAILURES.contains(&f.as_str()));

        assert!(
            failures.is_empty(),
            "Unexpected test failures: {:?}",
            failures
        );
    }
}

fn env_filter() -> EnvFilter {
    tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into())
}

pub type Action = Box<dyn Fn(&DataFlow) -> HandlerResult<DataFlowStatusMessage> + Send + Sync>;
pub struct TckTestHandler {
    actions: HashMap<String, Action>,
}

impl TckTestHandler {
    pub fn new() -> Self {
        let mut actions: HashMap<String, Action> = HashMap::new();

        actions.insert(
            "http_pull_sync".to_string(),
            Box::new(|flow| http_pull_sync(flow)),
        );
        actions.insert(
            "http_push_sync".to_string(),
            Box::new(|flow| http_push_sync(flow)),
        );
        Self { actions }
    }
}

#[async_trait::async_trait]
#[allow(unused_variables)]
impl DataFlowHandler for TckTestHandler {
    type Transaction = PgTransaction;
    async fn can_handle(&self, flow: &DataFlow) -> HandlerResult<bool> {
        Ok(true)
    }

    async fn on_start(
        &self,
        tx: &mut Self::Transaction,
        flow: &DataFlow,
    ) -> HandlerResult<DataFlowStatusMessage> {
        self.actions
            .get(&flow.transfer_type)
            .map(|action| action(flow))
            .ok_or_else(|| {
                HandlerError::NotSupported(format!(
                    "No action defined for transfer type: {}",
                    flow.transfer_type
                ))
            })?
    }

    async fn on_prepare(
        &self,
        tx: &mut Self::Transaction,
        flow: &DataFlow,
    ) -> HandlerResult<DataFlowStatusMessage> {
        self.actions
            .get(&flow.transfer_type)
            .map(|action| action(flow))
            .ok_or_else(|| {
                HandlerError::NotSupported(format!(
                    "No action defined for transfer type: {}",
                    flow.transfer_type
                ))
            })?
    }

    async fn on_terminate(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> HandlerResult<()> {
        Ok(())
    }

    async fn on_started(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> HandlerResult<()> {
        Ok(())
    }

    async fn on_suspend(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> HandlerResult<()> {
        Ok(())
    }
}

pub fn http_pull_sync(flow: &DataFlow) -> HandlerResult<DataFlowStatusMessage> {
    let (data_address, state) = match flow.kind {
        DataFlowType::Consumer => (None, DataFlowState::Prepared),
        DataFlowType::Provider => (
            Some(
                DataAddress::builder()
                    .endpoint("http://localhost:8282/async")
                    .endpoint_type("http")
                    .build(),
            ),
            DataFlowState::Started,
        ),
    };
    Ok(DataFlowStatusMessage::builder()
        .data_flow_id(flow.id.clone())
        .maybe_data_address(data_address)
        .state(state)
        .build())
}

pub fn http_push_sync(flow: &DataFlow) -> HandlerResult<DataFlowStatusMessage> {
    let (data_address, state) = match flow.kind {
        DataFlowType::Consumer => (
            Some(
                DataAddress::builder()
                    .endpoint("http://localhost:8282/async")
                    .endpoint_type("http")
                    .build(),
            ),
            DataFlowState::Prepared,
        ),
        DataFlowType::Provider => (None, DataFlowState::Started),
    };
    Ok(DataFlowStatusMessage::builder()
        .data_flow_id(flow.id.clone())
        .maybe_data_address(data_address)
        .state(state)
        .build())
}
