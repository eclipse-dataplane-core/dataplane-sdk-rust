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

use axum::{
    Extension, Json,
    extract::{Path, State},
};
use dataplane_sdk::{
    core::{
        db::tx::TransactionalContext,
        model::{
            messages::{
                DataFlowPrepareMessage, DataFlowStartMessage, DataFlowStartedNotificationMessage,
                DataFlowStatusMessage, DataFlowSuspendMessage, DataFlowTerminateMessage,
            },
            participant::ParticipantContext,
        },
    },
    sdk::DataPlaneSdk,
};
use serde::Deserialize;

use crate::error::SignalingResult;

pub async fn start_flow<C>(
    State(sdk): State<DataPlaneSdk<C>>,
    Extension(participant): Extension<ParticipantContext>,
    Json(msg): Json<DataFlowStartMessage>,
) -> SignalingResult<Json<DataFlowStatusMessage>>
where
    C: TransactionalContext,
{
    let response = sdk.start(&participant.id, msg).await?;
    Ok(Json(response))
}

pub async fn prepare_flow<C>(
    State(sdk): State<DataPlaneSdk<C>>,
    Extension(participant): Extension<ParticipantContext>,
    Json(msg): Json<DataFlowPrepareMessage>,
) -> SignalingResult<Json<DataFlowStatusMessage>>
where
    C: TransactionalContext,
{
    let response = sdk.prepare(&participant.id, msg).await?;
    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct FlowParams {
    #[allow(dead_code)]
    participant_context_id: Option<String>,
    id: String,
}
pub async fn started_flow<C>(
    State(sdk): State<DataPlaneSdk<C>>,
    Extension(participant): Extension<ParticipantContext>,
    Path(params): Path<FlowParams>,
    Json(msg): Json<DataFlowStartedNotificationMessage>,
) -> SignalingResult<()>
where
    C: TransactionalContext,
{
    sdk.started(&participant.id, &params.id, msg).await?;
    Ok(())
}

pub async fn terminate_flow<C>(
    State(sdk): State<DataPlaneSdk<C>>,
    Extension(participant): Extension<ParticipantContext>,
    Path(params): Path<FlowParams>,
    Json(msg): Json<DataFlowTerminateMessage>,
) -> SignalingResult<()>
where
    C: TransactionalContext,
{
    sdk.terminate(&participant.id, &params.id, msg.reason)
        .await?;
    Ok(())
}

pub async fn suspend_flow<C>(
    State(sdk): State<DataPlaneSdk<C>>,
    Extension(participant): Extension<ParticipantContext>,
    Path(params): Path<FlowParams>,
    Json(msg): Json<DataFlowSuspendMessage>,
) -> SignalingResult<()>
where
    C: TransactionalContext,
{
    sdk.suspend(&participant.id, &params.id, msg.reason).await?;
    Ok(())
}
