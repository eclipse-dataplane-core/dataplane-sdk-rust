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

use async_trait::async_trait;
use axum::{Router, routing::post};
use dataplane_sdk::core::error::HandlerResult;
use dataplane_sdk::core::model::messages::DataFlowResponseMessage;
use dataplane_sdk::{
    core::{
        db::{
            data_flow::DataFlowRepo,
            tx::{Transaction, TransactionalContext},
        },
        error::DbResult,
        handler::DataFlowHandler,
        model::data_flow::DataFlow,
    },
    sdk::DataPlaneSdk,
};
use mockall::mock;

use crate::api::{start_flow, suspend_flow, terminate_flow};

mock! {
    Tx{}

    #[async_trait]
    impl Transaction for Tx {
        async fn commit(self) -> DbResult<()>;
        async fn rollback(self) -> DbResult<()>;
    }
}

mock! {

    TxContext {}

    #[async_trait]
    impl TransactionalContext for TxContext {
        type Transaction = MockTx;
        async fn begin(&self) -> DbResult<MockTx>;
    }

}

mock! {
    Repo{}

    #[async_trait]
    impl DataFlowRepo for Repo {
        type Transaction = MockTx;
        async fn create(&self, tx: &mut MockTx, flow: &DataFlow) -> DbResult<()>;

        async fn update(&self, tx: &mut MockTx, flow: &DataFlow) -> DbResult<()>;

        async fn fetch_by_id(
            &self,
            tx: &mut MockTx,
            flow_id: &str,
        ) -> DbResult<Option<DataFlow>>;

        async fn delete(&self, tx: &mut MockTx, flow_id: &str) -> DbResult<()>;

    }
}

mock! {
    Handler {}

    #[async_trait]
    impl DataFlowHandler for Handler {
        type Transaction = MockTx;

        async fn can_handle(
            &self,
            flow: &DataFlow,
        ) -> HandlerResult<bool>;

        async fn on_start(
            &self,
            tx: &mut MockTx,
            flow: &DataFlow,
        ) -> HandlerResult<DataFlowResponseMessage>;

        async fn on_terminate(
            &self,
            tx: &mut MockTx,
            flow: &DataFlow,
        ) -> HandlerResult<()>;

        async fn on_prepare(
            &self,
            tx: &mut MockTx,
            flow: &DataFlow,
        ) -> HandlerResult<DataFlowResponseMessage>;

        async fn on_suspend(
            &self,
            tx: &mut MockTx,
            flow: &DataFlow,
        ) -> HandlerResult<()>;

        async fn on_started(
            &self,
            tx: &mut MockTx,
            flow: &DataFlow,
        ) -> HandlerResult<()>;
    }
}

fn app() -> Router<DataPlaneSdk<MockTxContext>> {
    Router::new()
        .route("/api/v1/dataflows/start", post(start_flow))
        .route("/api/v1/dataflows/{id}/terminate", post(terminate_flow))
        .route("/api/v1/dataflows/{id}/suspend", post(suspend_flow))
}

fn context() -> (MockTxContext, MockRepo, MockHandler) {
    (MockTxContext::new(), MockRepo::new(), MockHandler::new())
}

fn sdk(ctx: MockTxContext, repo: MockRepo, handler: MockHandler) -> DataPlaneSdk<MockTxContext> {
    DataPlaneSdk::builder(ctx)
        .with_repo(repo)
        .with_handler(handler)
        .build()
        .unwrap()
}

mod start {
    use axum::{
        Extension,
        body::Body,
        http::{Request, header::CONTENT_TYPE},
    };
    use dataplane_sdk::core::model::{
        data_address::DataAddress,
        data_flow::DataFlowState,
        messages::{DataFlowResponseMessage, DataFlowStartMessage},
        participant::ParticipantContext,
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::api_tests::{MockTx, app, context, sdk};

    #[tokio::test]
    async fn start_flow_test() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTx::new();
            tx.expect_commit().returning(|| Ok(()));
            Ok(tx)
        });

        repo.expect_create().returning(|_, _| Ok(()));

        handler.expect_can_handle().returning(|_| Ok(true));
        handler.expect_on_start().returning(|_, _flow| {
            Ok(DataFlowResponseMessage::builder()
                .state(DataFlowState::Started)
                .dataplane_id("dataplane_id")
                .build())
        });

        let p_context = ParticipantContext::builder()
            .id("example-participant")
            .build();

        let app = app()
            .with_state(sdk(ctx, repo, handler))
            .layer(Extension(p_context));

        let msg = DataFlowStartMessage::builder()
            .dataset_id("dataset_id")
            .participant_id("counter_party_id")
            .process_id("process_id")
            .data_address(
                DataAddress::builder()
                    .endpoint_type("type")
                    .endpoint("endpoint")
                    .build(),
            )
            .agreement_id("agreement_id")
            .transfer_type("transfer_type")
            .dataspace_context("dataspace_context")
            .callback_address("callback_address")
            .message_id("message_id")
            .counter_party_id("counter_party_id")
            .build();

        let payload = serde_json::to_vec(&msg).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header(CONTENT_TYPE, "application/json")
                    .uri("/api/v1/dataflows/start")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();

        let body: DataFlowResponseMessage = serde_json::from_slice(&body).unwrap();

        assert!(body.data_address.is_none());
    }
}

mod terminate {

    use axum::{
        Extension,
        body::Body,
        http::{Request, header::CONTENT_TYPE},
    };
    use dataplane_sdk::core::model::{
        data_flow::{DataFlow, DataFlowState, DataFlowType},
        messages::DataFlowTerminateMessage,
        participant::ParticipantContext,
    };
    use tower::ServiceExt;

    use crate::api_tests::{MockTx, app, context, sdk};

    #[tokio::test]
    async fn terminate_test() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTx::new();
            tx.expect_commit().returning(|| Ok(()));
            Ok(tx)
        });

        repo.expect_fetch_by_id().returning(|_, _| {
            Ok(Some(
                DataFlow::builder()
                    .id("example-flow-id")
                    .state(DataFlowState::Started)
                    .counter_party_id("counter_party_id")
                    .participant_context_id("example-participant")
                    .dataset_id("dataset_id")
                    .agreement_id("agreement_id")
                    .dataspace_context("dataspace_context")
                    .participant_id("participant_id")
                    .callback_address("callback_address")
                    .transfer_type("transfer_type")
                    .kind(DataFlowType::Provider)
                    .build(),
            ))
        });
        repo.expect_update().returning(|_, _| Ok(()));

        handler.expect_on_terminate().returning(|_, _flow| Ok(()));

        let p_context = ParticipantContext::builder()
            .id("example-participant")
            .build();

        let app = app()
            .with_state(sdk(ctx, repo, handler))
            .layer(Extension(p_context));

        let msg = DataFlowTerminateMessage::builder()
            .reason("dataset_id")
            .build();

        let payload = serde_json::to_vec(&msg).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header(CONTENT_TYPE, "application/json")
                    .uri(format!("/api/v1/dataflows/{}/terminate", "example-flow-id"))
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(
            response.status().is_success(),
            "Response was not successful: {:?}",
            response.status()
        );
    }
}

mod suspend {

    use axum::{
        Extension,
        body::Body,
        http::{Request, header::CONTENT_TYPE},
    };
    use dataplane_sdk::core::model::{
        data_flow::{DataFlow, DataFlowState, DataFlowType},
        messages::DataFlowTerminateMessage,
        participant::ParticipantContext,
    };
    use tower::ServiceExt;

    use crate::api_tests::{MockTx, app, context, sdk};

    #[tokio::test]
    async fn suspend_test() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTx::new();
            tx.expect_commit().returning(|| Ok(()));
            Ok(tx)
        });

        repo.expect_fetch_by_id().returning(|_, _| {
            Ok(Some(
                DataFlow::builder()
                    .id("example-flow-id")
                    .state(DataFlowState::Started)
                    .counter_party_id("counter_party_id")
                    .participant_context_id("example-participant")
                    .dataset_id("dataset_id")
                    .agreement_id("agreement_id")
                    .dataspace_context("dataspace_context")
                    .participant_id("participant_id")
                    .callback_address("callback_address")
                    .transfer_type("transfer_type")
                    .kind(DataFlowType::Provider)
                    .build(),
            ))
        });
        repo.expect_update().returning(|_, _| Ok(()));

        handler.expect_on_suspend().returning(|_, _flow| Ok(()));

        let p_context = ParticipantContext::builder()
            .id("example-participant")
            .build();

        let app = app()
            .with_state(sdk(ctx, repo, handler))
            .layer(Extension(p_context));

        let msg = DataFlowTerminateMessage::builder()
            .reason("dataset_id")
            .build();

        let payload = serde_json::to_vec(&msg).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header(CONTENT_TYPE, "application/json")
                    .uri(format!("/api/v1/dataflows/{}/suspend", "example-flow-id"))
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(
            response.status().is_success(),
            "Response was not successful: {:?}",
            response.status()
        );
    }
}
