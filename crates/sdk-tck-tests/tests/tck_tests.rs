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

use std::{collections::HashMap, time::Duration};

use bon::Builder;
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
use tokio::sync::mpsc::Sender;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

mod util;

#[cfg(test)]
mod tck_tests {
    use dataplane_sdk::{core::db::tx::TransactionalContext, sdk::DataPlaneSdk};
    use tokio::sync::mpsc::Receiver;

    use crate::util::TckTestReporter;

    use super::*;

    fn handle_notifications<T: TransactionalContext + 'static>(
        sdk: DataPlaneSdk<T>,
        rx: Receiver<Notification>,
    ) where
        <T as TransactionalContext>::Transaction: std::marker::Send,
    {
        tokio::task::spawn(async move {
            let mut stream = rx;
            while let Some(notification) = stream.recv().await {
                tokio::time::sleep(Duration::from_millis(250)).await;
                let flow = &notification.flow;
                match notification.kind {
                    NotificationKind::Started => {
                        sdk.notify_started(&flow.participant_context_id, &flow.id, None)
                            .await
                            .unwrap();
                    }
                    NotificationKind::Suspended => tracing::info!(
                        "Flow suspended: ID: {}, state: {:?}",
                        notification.flow.id,
                        notification.flow.state
                    ),
                    NotificationKind::Completed => {
                        sdk.notify_completed(&flow.participant_context_id, &flow.id)
                            .await
                            .unwrap();
                    }
                    NotificationKind::Errored(err) => tracing::error!(
                        "Flow errored: ID: {}, state: {:?}, error: {}",
                        notification.flow.id,
                        notification.flow.state,
                        err
                    ),
                    NotificationKind::Prepared => {
                        sdk.notify_prepared(&flow.participant_context_id, &flow.id, None)
                            .await
                            .unwrap();
                    }
                }
            }
        });
    }

    #[tokio::test]
    async fn dataplane_tck_test() {
        tracing_subscriber::registry()
            .with(env_filter())
            .with(tracing_subscriber::fmt::layer())
            .init();
        let (ctx, repo, _container) = util::setup_postgres_container().await;

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let sdk = util::sdk(ctx, repo, TckTestHandler::new(tx)).await;

        handle_notifications(sdk.clone(), rx);

        util::start_signaling(8282, sdk).await;

        let reporter = TckTestReporter::default();

        let _tck_container = util::setup_tck_container(reporter.clone()).await;

        let failures = reporter.failures();

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

pub type Handler = Box<dyn Fn(&DataFlow) -> HandlerResult<DataFlowStatusMessage> + Send + Sync>;

#[derive(Builder, Clone)]
pub struct Notification {
    flow: DataFlow,
    kind: NotificationKind,
}

#[derive(Clone, Debug)]
pub enum NotificationKind {
    Started,
    Suspended,
    Completed,
    Prepared,
    Errored(String),
}
pub struct TckTestHandler {
    handlers: HashMap<String, Handler>,
    sender: Sender<Notification>,
}

impl TckTestHandler {
    pub fn new(sender: Sender<Notification>) -> Self {
        let mut handlers: HashMap<String, Handler> = HashMap::new();

        handlers.insert(
            "http_pull_sync".to_string(),
            Box::new(|flow| http_pull_sync(flow)),
        );
        handlers.insert(
            "http_push_sync".to_string(),
            Box::new(|flow| http_push_sync(flow)),
        );

        handlers.insert(
            "http_pull_async".to_string(),
            Box::new(|flow| http_pull_async(flow)),
        );

        handlers.insert(
            "http_push_async".to_string(),
            Box::new(|flow| http_push_async(flow)),
        );
        Self { handlers, sender }
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
        self.fire_notification(flow).await;
        self.handlers
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
        self.fire_notification(flow).await;
        self.handlers
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
        self.fire_notification(flow).await;
        Ok(())
    }

    async fn on_started(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> HandlerResult<()> {
        self.fire_notification(flow).await;
        Ok(())
    }

    async fn on_suspend(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> HandlerResult<()> {
        self.fire_notification(flow).await;
        Ok(())
    }
}

fn matches_state(state: &str, expected: DataFlowState) -> bool {
    match expected {
        DataFlowState::Prepared => state == "prepared",
        DataFlowState::Started => state == "started",
        DataFlowState::Initiating => state == "initiating",
        _ => false,
    }
}

impl TckTestHandler {
    async fn fire_notification(&self, flow: &DataFlow) {
        let notification = flow.agreement_id.split("-").collect::<Vec<&str>>();
        let kind = match notification.as_slice() {
            [state, "completed"] if matches_state(state, flow.state.clone()) => {
                NotificationKind::Completed
            }
            [state, "prepared"] if matches_state(state, flow.state.clone()) => {
                NotificationKind::Prepared
            }
            [state, "started"] if matches_state(state, flow.state.clone()) => {
                NotificationKind::Started
            }
            [state, "error"] if matches_state(state, flow.state.clone()) => {
                NotificationKind::Errored("Simulated error".to_string())
            }
            _ => return, // No notification for other agreement IDs
        };
        let notification = Notification::builder()
            .flow(flow.clone())
            .kind(kind)
            .build();

        self.sender.send(notification).await.unwrap();
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

pub fn http_pull_async(flow: &DataFlow) -> HandlerResult<DataFlowStatusMessage> {
    let (data_address, state) = match flow.kind {
        DataFlowType::Consumer => (None, DataFlowState::Preparing),
        DataFlowType::Provider => (None, DataFlowState::Starting),
    };
    Ok(DataFlowStatusMessage::builder()
        .data_flow_id(flow.id.clone())
        .maybe_data_address(data_address)
        .state(state)
        .build())
}

pub fn http_push_async(flow: &DataFlow) -> HandlerResult<DataFlowStatusMessage> {
    let (data_address, state) = match flow.kind {
        DataFlowType::Consumer => (None, DataFlowState::Preparing),
        DataFlowType::Provider => (None, DataFlowState::Starting),
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
