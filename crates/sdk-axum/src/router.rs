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

use axum::{Router, routing::post};
use dataplane_sdk::{core::db::tx::TransactionalContext, sdk::DataPlaneSdk};

use crate::api::{prepare_flow, start_flow, started_flow, suspend_flow, terminate_flow};

pub fn router<C>() -> Router<DataPlaneSdk<C>>
where
    C: TransactionalContext + 'static,
    C::Transaction: Send,
{
    Router::new()
        .route("/api/v1/dataflows/start", post(start_flow))
        .route("/api/v1/dataflows/prepare", post(prepare_flow))
        .route("/api/v1/dataflows/{id}/terminate", post(terminate_flow))
        .route("/api/v1/dataflows/{id}/started", post(started_flow))
        .route("/api/v1/dataflows/{id}/suspend", post(suspend_flow))
}

pub fn participants_router<C>() -> Router<DataPlaneSdk<C>>
where
    C: TransactionalContext + 'static,
    C::Transaction: Send,
{
    Router::new()
        .route(
            "/api/v1/{participant_context_id}/dataflows/start",
            post(start_flow),
        )
        .route(
            "/api/v1/{participant_context_id}/dataflows/prepare",
            post(prepare_flow),
        )
        .route(
            "/api/v1/{participant_context_id}/dataflows/{id}/terminate",
            post(terminate_flow),
        )
        .route(
            "/api/v1/{participant_context_id}/dataflows/{id}/started",
            post(started_flow),
        )
        .route(
            "/api/v1/{participant_context_id}/dataflows/{id}/suspend",
            post(suspend_flow),
        )
}
