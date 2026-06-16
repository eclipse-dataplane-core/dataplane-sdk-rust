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

use crate::core::{
    db::{data_flow::MockDataFlowRepo, tx::MockTransactionalContext},
    handler::MockDataFlowHandler,
    model::{
        data_address::DataAddress,
        data_flow::{DataFlow, DataFlowState, DataFlowType},
        messages::{DataFlowPrepareMessage, DataFlowStartMessage},
    },
};

mod prepare {

    use std::future;

    use crate::{
        core::{
            db::tx::MockTransaction,
            error::{DbError, HandlerError},
            model::{data_flow::DataFlowState, messages::DataFlowStatusMessage},
        },
        error::SdkError,
        sdk::DataPlaneSdk,
        sdk_test::{context, prepare_message},
    };

    #[tokio::test]
    async fn prepare() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_create()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        handler.expect_on_prepare().returning(|_, _| {
            Box::pin(future::ready(Ok(DataFlowStatusMessage::builder()
                .state(DataFlowState::Prepared)
                .data_flow_id("flow-id")
                .build())))
        });

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.prepare("participant", prepare_message()).await.unwrap();

        assert!(response.data_address.is_none());
    }

    #[tokio::test]
    async fn prepare_begin_fails() {
        let (mut ctx, repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            Box::pin(future::ready(Err(DbError::Generic(
                "Begin failed".to_string().into(),
            ))))
        });

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.prepare("participant", prepare_message()).await;

        assert!(matches!(response, Err(SdkError::Repo(DbError::Generic(_)))));
    }

    #[tokio::test]
    async fn prepare_create_fails() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin()
            .returning(|| Box::pin(future::ready(Ok(MockTransaction::new()))));

        repo.expect_create().returning(|_, _| {
            Box::pin(future::ready(Err(DbError::AlreadyExists(
                "Data flow already exists".to_string(),
            ))))
        });

        handler.expect_on_prepare().returning(|_, _| {
            Box::pin(future::ready(Ok(DataFlowStatusMessage::builder()
                .data_flow_id("flow-id")
                .state(DataFlowState::Prepared)
                .build())))
        });

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.prepare("participant", prepare_message()).await;

        assert!(matches!(
            response,
            Err(SdkError::Repo(DbError::AlreadyExists(_)))
        ));
    }

    #[tokio::test]
    async fn prepare_handler_fails() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_create()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        handler.expect_on_prepare().returning(|_, _| {
            Box::pin(future::ready(Err(HandlerError::Generic(
                "Handler error".to_string().into(),
            ))))
        });

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.prepare("participant", prepare_message()).await;

        assert!(matches!(
            response,
            Err(SdkError::Handler(HandlerError::Generic(_)))
        ));
    }

    #[tokio::test]
    async fn prepare_handler_not_supported() {
        let (ctx, repo, mut handler) = context();

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(false))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.prepare("participant", prepare_message()).await;

        assert!(matches!(
            response,
            Err(SdkError::Handler(HandlerError::NotSupported(_)))
        ));
    }
}

mod start {

    use std::future;

    use crate::{
        core::{
            db::tx::MockTransaction,
            error::{DbError, HandlerError},
            model::{data_flow::DataFlowState, messages::DataFlowStatusMessage},
        },
        error::SdkError,
        sdk::DataPlaneSdk,
        sdk_test::{context, start_message},
    };

    #[tokio::test]
    async fn start() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_create()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        handler.expect_on_start().returning(|_, _| {
            Box::pin(future::ready(Ok(DataFlowStatusMessage::builder()
                .data_flow_id("flow-id")
                .state(DataFlowState::Started)
                .build())))
        });

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.start("participant", start_message()).await.unwrap();

        assert!(response.data_address.is_none());
    }

    #[tokio::test]
    async fn start_begin_fails() {
        let (mut ctx, repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            Box::pin(future::ready(Err(DbError::Generic(
                "Begin failed".to_string().into(),
            ))))
        });

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.start("participant", start_message()).await;

        assert!(matches!(response, Err(SdkError::Repo(DbError::Generic(_)))));
    }

    #[tokio::test]
    async fn start_create_fails() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin()
            .returning(|| Box::pin(future::ready(Ok(MockTransaction::new()))));

        repo.expect_create().returning(|_, _| {
            Box::pin(future::ready(Err(DbError::AlreadyExists(
                "Data flow already exists".to_string(),
            ))))
        });

        handler.expect_on_start().returning(|_, _| {
            Box::pin(future::ready(Ok(DataFlowStatusMessage::builder()
                .data_flow_id("flow-id")
                .state(DataFlowState::Started)
                .build())))
        });

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.start("participant", start_message()).await;

        assert!(matches!(
            response,
            Err(SdkError::Repo(DbError::AlreadyExists(_)))
        ));
    }

    #[tokio::test]
    async fn start_handler_fails() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_create()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        handler.expect_on_start().returning(|_, _| {
            Box::pin(future::ready(Err(HandlerError::Generic(
                "Handler error".to_string().into(),
            ))))
        });

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.start("participant", start_message()).await;

        assert!(matches!(
            response,
            Err(SdkError::Handler(HandlerError::Generic(_)))
        ));
    }

    #[tokio::test]
    async fn start_handler_not_supported() {
        let (ctx, repo, mut handler) = context();

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(false))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.start("participant", start_message()).await;

        assert!(matches!(
            response,
            Err(SdkError::Handler(HandlerError::NotSupported(_)))
        ));
    }
}

mod terminate {
    use std::future;

    use crate::{
        core::{db::tx::MockTransaction, error::DbError},
        sdk::DataPlaneSdk,
        sdk_test::{context, flow},
    };

    #[tokio::test]
    async fn terminate() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(Some(flow())))));

        repo.expect_update()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        handler
            .expect_on_terminate()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.terminate("participant", "flow_id", None).await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn terminate_not_found() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(None))));

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        handler.expect_on_terminate().times(0);

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.terminate("participant", "flow_id", None).await;

        assert!(matches!(
            response,
            Err(crate::error::SdkError::Repo(DbError::NotFound(_)))
        ));
    }
}

mod suspend {
    use std::future;

    use crate::{
        core::{db::tx::MockTransaction, error::DbError},
        sdk::DataPlaneSdk,
        sdk_test::{context, flow},
    };

    #[tokio::test]
    async fn suspend() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(Some(flow())))));

        repo.expect_update()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        handler
            .expect_on_suspend()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.suspend("participant", "flow_id", None).await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn suspend_not_found() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(None))));

        handler
            .expect_can_handle()
            .returning(|_| Box::pin(future::ready(Ok(true))));

        handler.expect_on_suspend().times(0);

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.suspend("participant", "flow_id", None).await;

        assert!(matches!(
            response,
            Err(crate::error::SdkError::Repo(DbError::NotFound(_)))
        ));
    }
}

mod started {
    use std::future;

    use crate::{
        core::{
            db::tx::MockTransaction, error::DbError,
            model::messages::DataFlowStartedNotificationMessage,
        },
        sdk::DataPlaneSdk,
        sdk_test::{context, flow},
    };

    fn started_message() -> DataFlowStartedNotificationMessage {
        DataFlowStartedNotificationMessage::builder().build()
    }

    #[tokio::test]
    async fn started() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(Some(flow())))));

        repo.expect_update()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        handler
            .expect_on_started()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk
            .started("participant", "flow_id", started_message())
            .await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn started_not_found() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(None))));

        handler.expect_on_started().times(0);

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk
            .started("participant", "flow_id", started_message())
            .await;

        assert!(matches!(
            response,
            Err(crate::error::SdkError::Repo(DbError::NotFound(_)))
        ));
    }
}

mod completed {
    use std::future;

    use crate::{
        core::{db::tx::MockTransaction, error::DbError},
        sdk::DataPlaneSdk,
        sdk_test::{context, flow},
    };

    #[tokio::test]
    async fn completed() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(Some(flow())))));

        repo.expect_update()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        handler
            .expect_on_completed()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.completed("participant", "flow_id").await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn completed_not_found() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(None))));

        handler.expect_on_completed().times(0);

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.completed("participant", "flow_id").await;

        assert!(matches!(
            response,
            Err(crate::error::SdkError::Repo(DbError::NotFound(_)))
        ));
    }
}

mod resume {
    use std::future;

    use crate::{
        core::{
            db::tx::MockTransaction,
            error::DbError,
            model::{
                data_flow::DataFlowState,
                messages::{DataFlowResumeMessage, DataFlowStatusMessage},
            },
        },
        sdk::DataPlaneSdk,
        sdk_test::{context, flow},
    };

    fn resume_message() -> DataFlowResumeMessage {
        DataFlowResumeMessage::builder().build()
    }

    #[tokio::test]
    async fn resume() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id().returning(|_, _| {
            let mut f = flow();
            f.state = DataFlowState::Suspended;
            Box::pin(future::ready(Ok(Some(f))))
        });

        repo.expect_update()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        handler.expect_on_resume().returning(|_, _| {
            Box::pin(future::ready(Ok(DataFlowStatusMessage::builder()
                .data_flow_id("flow-id")
                .state(DataFlowState::Started)
                .build())))
        });

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk
            .resume("participant", "flow_id", resume_message())
            .await
            .unwrap();

        assert!(matches!(response.state, DataFlowState::Started));
    }

    #[tokio::test]
    async fn resume_not_found() {
        let (mut ctx, mut repo, mut handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(None))));

        handler.expect_on_resume().times(0);

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.resume("participant", "flow_id", resume_message()).await;

        assert!(matches!(
            response,
            Err(crate::error::SdkError::Repo(DbError::NotFound(_)))
        ));
    }
}

mod status {
    use std::future;

    use crate::{
        core::{db::tx::MockTransaction, error::DbError, model::data_flow::DataFlowState},
        sdk::DataPlaneSdk,
        sdk_test::{context, flow},
    };

    #[tokio::test]
    async fn status() {
        let (mut ctx, mut repo, handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(Some(flow())))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.status("participant", "flow_id").await.unwrap();

        assert_eq!(response.data_flow_id, "flow-id");
        assert!(matches!(response.state, DataFlowState::Started));
    }

    #[tokio::test]
    async fn status_not_found() {
        let (mut ctx, mut repo, handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(None))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.status("participant", "flow_id").await;

        assert!(matches!(
            response,
            Err(crate::error::SdkError::Repo(DbError::NotFound(_)))
        ));
    }
}

mod notify {
    use std::future;

    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{body_string_contains, method, path},
    };

    use crate::{
        core::{
            db::tx::{MockTransaction, MockTransactionalContext},
            error::DbError,
            model::data_flow::{DataFlow, DataFlowState},
        },
        error::SdkError,
        sdk::DataPlaneSdk,
        sdk_test::{context, flow},
    };

    fn flow_with(callback: &str, state: DataFlowState) -> DataFlow {
        let mut f = flow();
        f.callback_address = callback.to_string();
        f.state = state;
        f
    }

    /// Configures the mock context so that `begin`/`commit` succeed and
    /// `fetch_by_id` returns the supplied flow.
    fn with_flow(f: DataFlow) -> DataPlaneSdk<MockTransactionalContext> {
        let (mut ctx, mut repo, handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(move |_, _| Box::pin(future::ready(Ok(Some(f.clone())))));

        repo.expect_update()
            .returning(|_, _| Box::pin(future::ready(Ok(()))));

        DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn notify_started_posts_to_callback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/transfers/flow-id/dataflow/started"))
            .and(body_string_contains("\"state\":\"STARTED\""))
            .and(body_string_contains("\"dataFlowId\":\"flow-id\""))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let sdk = with_flow(flow_with(&server.uri(), DataFlowState::Started));

        let response = sdk.notify_started("participant", "flow-id", None).await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn notify_prepared_posts_to_callback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/transfers/flow-id/dataflow/prepared"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let sdk = with_flow(flow_with(&server.uri(), DataFlowState::Prepared));

        assert!(
            sdk.notify_prepared("participant", "flow-id", None)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn notify_completed_posts_to_callback() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/transfers/flow-id/dataflow/completed"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let sdk = with_flow(flow_with(&server.uri(), DataFlowState::Completed));

        assert!(sdk.notify_completed("participant", "flow-id").await.is_ok());
    }

    #[tokio::test]
    async fn notify_errored_forwards_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/transfers/flow-id/dataflow/errored"))
            .and(body_string_contains("something broke"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let sdk = with_flow(flow_with(&server.uri(), DataFlowState::Started));

        let response = sdk
            .notify_errored(
                "participant",
                "flow-id",
                Some("something broke".to_string()),
            )
            .await;

        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn notify_non_success_status_is_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
            .mount(&server)
            .await;

        let sdk = with_flow(flow_with(&server.uri(), DataFlowState::Started));

        let response = sdk.notify_started("participant", "flow-id", None).await;

        assert!(matches!(
            response,
            Err(SdkError::NotificationStatus { status: 500, .. })
        ));
    }

    #[tokio::test]
    async fn notify_not_found() {
        let (mut ctx, mut repo, handler) = context();

        ctx.expect_begin().returning(|| {
            let mut tx = MockTransaction::new();
            tx.expect_commit()
                .returning(|| Box::pin(future::ready(Ok(()))));
            Box::pin(future::ready(Ok(tx)))
        });

        repo.expect_fetch_by_id()
            .returning(|_, _| Box::pin(future::ready(Ok(None))));

        let sdk = DataPlaneSdk::builder(ctx)
            .with_repo(repo)
            .with_handler(handler)
            .build()
            .unwrap();

        let response = sdk.notify_started("participant", "flow-id", None).await;

        assert!(matches!(
            response,
            Err(SdkError::Repo(DbError::NotFound(_)))
        ));
    }
}

fn start_message() -> DataFlowStartMessage {
    DataFlowStartMessage::builder()
        .process_id("process-id")
        .participant_id("counter-party")
        .dataset_id("dataset")
        .data_address(
            DataAddress::builder()
                .endpoint_type("Type")
                .endpoint("endpoint")
                .build(),
        )
        .agreement_id("agreement")
        .callback_address("callback")
        .transfer_type("transfer-type")
        .counter_party_id("counter-party")
        .dataspace_context("dataspace-context")
        .message_id("message-id")
        .build()
}

fn prepare_message() -> DataFlowPrepareMessage {
    DataFlowPrepareMessage::builder()
        .process_id("process-id")
        .participant_id("counter-party")
        .dataset_id("dataset")
        .agreement_id("agreement")
        .callback_address("callback")
        .transfer_type("transfer-type")
        .counter_party_id("counter-party")
        .dataspace_context("dataspace-context")
        .message_id("message-id")
        .build()
}

fn flow() -> DataFlow {
    DataFlow::builder()
        .id("flow-id")
        .state(DataFlowState::Started)
        .participant_context_id("participant-1")
        .counter_party_id("counter-party")
        .data_address(
            DataAddress::builder()
                .endpoint_type("Type")
                .endpoint("endpoint")
                .build(),
        )
        .dataset_id("dataset")
        .agreement_id("agreement")
        .callback_address("callback")
        .transfer_type("transfer-type")
        .dataspace_context("dataspace-context")
        .labels(vec![])
        .participant_id("participant")
        .kind(DataFlowType::Provider)
        .build()
}

fn context() -> (
    MockTransactionalContext,
    MockDataFlowRepo,
    MockDataFlowHandler,
) {
    (
        MockTransactionalContext::new(),
        MockDataFlowRepo::new(),
        MockDataFlowHandler::new(),
    )
}
