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
                .state(DataFlowState::Started)
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
