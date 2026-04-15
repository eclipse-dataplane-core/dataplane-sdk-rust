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
use axum::Router;
use dataplane_sdk::core::error::HandlerResult;
use dataplane_sdk::core::model::messages::DataFlowStatusMessage;
use dataplane_sdk::core::model::participant::ParticipantContext;
use dataplane_sdk::sdk;
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

use crate::router::{participants_router, router};

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
        ) -> HandlerResult<DataFlowStatusMessage>;

        async fn on_terminate(
            &self,
            tx: &mut MockTx,
            flow: &DataFlow,
        ) -> HandlerResult<()>;

        async fn on_prepare(
            &self,
            tx: &mut MockTx,
            flow: &DataFlow,
        ) -> HandlerResult<DataFlowStatusMessage>;

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

struct TestCtx {
    base_url: String,
    app: Router<DataPlaneSdk<MockTxContext>>,
    participant_context: ParticipantContext,
}

impl TestCtx {
    pub fn app(&self, sdk: DataPlaneSdk<MockTxContext>) -> Router {
        let app = self.app.clone().with_state(sdk);
        app.layer(axum::Extension(self.participant_context.clone()))
    }
}

fn single_ctx() -> TestCtx {
    TestCtx {
        base_url: "/api/v1".to_string(),
        app: router(),
        participant_context: ParticipantContext::builder()
            .id("example-participant")
            .build(),
    }
}

fn multi_participant_ctx() -> TestCtx {
    TestCtx {
        base_url: "/api/v1/example-participant".to_string(),
        app: participants_router(),
        participant_context: ParticipantContext::builder()
            .id("example-participant")
            .build(),
    }
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
        body::Body,
        http::{Request, header::CONTENT_TYPE},
    };
    use dataplane_sdk::core::model::{
        data_address::DataAddress,
        data_flow::DataFlowState,
        messages::{DataFlowStartMessage, DataFlowStatusMessage},
    };
    use http_body_util::BodyExt;
    use rstest::rstest;
    use tower::ServiceExt;

    use crate::api_tests::{MockTx, TestCtx, context, multi_participant_ctx, sdk, single_ctx};

    #[rstest]
    #[case(single_ctx())]
    #[case(multi_participant_ctx())]
    #[tokio::test]
    async fn start_flow_test(#[case] test_ctx: TestCtx) {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTx::new();
            tx.expect_commit().returning(|| Ok(()));
            Ok(tx)
        });

        repo.expect_create().returning(|_, _| Ok(()));

        handler.expect_can_handle().returning(|_| Ok(true));
        handler.expect_on_start().returning(|_, _flow| {
            Ok(DataFlowStatusMessage::builder()
                .state(DataFlowState::Started)
                .build())
        });

        let app = test_ctx.app(sdk(ctx, repo, handler));

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
                    .uri(test_ctx.base_url + "/dataflows/start")
                    .body(Body::from(payload))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();

        let body: DataFlowStatusMessage = serde_json::from_slice(&body).unwrap();

        assert!(body.data_address.is_none());
    }
}

mod terminate {

    use axum::{
        body::Body,
        http::{Request, header::CONTENT_TYPE},
    };
    use dataplane_sdk::core::model::{
        data_flow::{DataFlow, DataFlowState, DataFlowType},
        messages::DataFlowTerminateMessage,
    };
    use rstest::rstest;
    use tower::ServiceExt;

    use crate::api_tests::{MockTx, TestCtx, context, multi_participant_ctx, sdk, single_ctx};

    #[rstest]
    #[case(single_ctx())]
    #[case(multi_participant_ctx())]
    #[tokio::test]
    async fn terminate_test(#[case] test_ctx: TestCtx) {
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

        let app = test_ctx.app(sdk(ctx, repo, handler));

        let msg = DataFlowTerminateMessage::builder()
            .reason("dataset_id")
            .build();

        let payload = serde_json::to_vec(&msg).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header(CONTENT_TYPE, "application/json")
                    .uri(format!(
                        "{}/dataflows/{}/terminate",
                        test_ctx.base_url, "example-flow-id"
                    ))
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
        body::Body,
        http::{Request, header::CONTENT_TYPE},
    };

    use dataplane_sdk::core::model::{
        data_flow::{DataFlow, DataFlowState, DataFlowType},
        messages::DataFlowTerminateMessage,
    };
    use rstest::rstest;
    use tower::ServiceExt;

    use crate::api_tests::{MockTx, TestCtx, context, multi_participant_ctx, sdk, single_ctx};

    #[rstest]
    #[case(single_ctx())]
    #[case(multi_participant_ctx())]
    #[tokio::test]
    async fn suspend_test(#[case] test_ctx: TestCtx) {
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

        let app = test_ctx.app(sdk(ctx, repo, handler));

        let msg = DataFlowTerminateMessage::builder()
            .reason("dataset_id")
            .build();

        let payload = serde_json::to_vec(&msg).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .header(CONTENT_TYPE, "application/json")
                    .uri(format!(
                        "{}/dataflows/{}/suspend",
                        test_ctx.base_url, "example-flow-id"
                    ))
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
