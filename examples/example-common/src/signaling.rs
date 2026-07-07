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

use std::{net::SocketAddr, str::FromStr, sync::Arc};

use axum::Extension;
use dataplane_sdk::{
    core::{
        db::tx::TransactionalContext,
        model::{control_plane::ControlPlane, participant::ParticipantContext},
    },
    sdk::DataPlaneSdk,
};
use dataplane_sdk_axum::router::router;
use tokio::sync::Barrier;

use crate::util::launch_server;

pub async fn start_signaling<C>(
    port: u16,
    sdk: DataPlaneSdk<C>,
    control_plane: ControlPlane,
    barrier: Arc<Barrier>,
) where
    C: TransactionalContext + 'static,
    C::Transaction: Send,
{
    let addr = SocketAddr::from_str(&format!("0.0.0.0:{port}")).expect("Invalid socket address");

    let p_context = ParticipantContext::builder()
        .id("example-participant")
        .build();
    let router = router()
        .layer(Extension(p_context))
        .layer(Extension(control_plane));

    launch_server("Signaling API", router, sdk.clone(), addr, barrier).await;
}
