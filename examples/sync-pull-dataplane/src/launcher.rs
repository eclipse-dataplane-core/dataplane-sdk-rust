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

use crate::api::{public::start_public_api, tokens::start_token_api};
use crate::config::DataPlaneConfig;
use crate::handler::TokenHandler;
use crate::tokens::manager::TokenManager;
use crate::tokens::repo::TokenRepo;
use crate::tokens::repo::memory::MemoryTokenRepo;
use crate::tokens::repo::postgres::PgTokenRepo;
use dataplane_sdk::core::db::control_plane::ControlPlaneRepo;
use dataplane_sdk::core::db::control_plane::memory::MemoryControlPlaneRepo;
use dataplane_sdk::core::db::data_flow::DataFlowRepo;
use dataplane_sdk::core::db::data_flow::memory::MemoryDataFlowRepo;
use dataplane_sdk::core::db::memory::MemoryContext;
use dataplane_sdk::core::db::tx::{Transaction, TransactionalContext};
use dataplane_sdk::core::model::control_plane::ControlPlane;
use dataplane_sdk::sdk::DataPlaneSdk;
use dataplane_sdk_postgres::{PgContext, PgControlPlaneRepo, PgDataFlowRepo};
use example_common::controlplane::CONTROL_PLANE_ID;
use example_common::signaling::start_signaling;
use tokio::sync::Barrier;

pub async fn start_dataplane(cfg: DataPlaneConfig) -> anyhow::Result<()> {
    match &cfg.db {
        crate::config::Db::Memory => {
            let (ctx, repo, control_plane_repo, token_repo) = setup_memory().await?;
            internal_launch(&cfg, ctx, repo, control_plane_repo, token_repo).await
        }
        crate::config::Db::Postgres { url } => {
            let (ctx, repo, control_plane_repo, token_repo) = setup_pg(url).await?;
            internal_launch(&cfg, ctx, repo, control_plane_repo, token_repo).await
        }
    }
}

async fn setup_memory() -> anyhow::Result<(
    MemoryContext,
    MemoryDataFlowRepo,
    MemoryControlPlaneRepo,
    MemoryTokenRepo,
)> {
    let ctx = MemoryContext;

    let repo = MemoryDataFlowRepo::default();
    let control_plane_repo = MemoryControlPlaneRepo::default();
    let token_repo = MemoryTokenRepo::default();

    Ok((ctx, repo, control_plane_repo, token_repo))
}

async fn setup_pg(
    url: &str,
) -> anyhow::Result<(PgContext, PgDataFlowRepo, PgControlPlaneRepo, PgTokenRepo)> {
    let ctx = PgContext::connect(url).await?;

    let mut tx = ctx.begin().await?;
    let repo = PgDataFlowRepo;
    let control_plane_repo = PgControlPlaneRepo;
    let token_repo = PgTokenRepo;

    repo.migrate(&mut tx).await?;
    control_plane_repo.migrate(&mut tx).await?;
    token_repo.migrate(&mut tx).await?;

    tx.commit().await?;

    Ok((ctx, repo, control_plane_repo, token_repo))
}

async fn internal_launch<C, R, CP, T>(
    cfg: &DataPlaneConfig,
    ctx: C,
    flows: R,
    control_planes: CP,
    tokens: T,
) -> anyhow::Result<()>
where
    C: TransactionalContext + 'static,
    C::Transaction: Send,
    R: DataFlowRepo<Transaction = C::Transaction> + 'static,
    CP: ControlPlaneRepo<Transaction = C::Transaction> + 'static,
    T: TokenRepo<Transaction = C::Transaction> + 'static,
{
    let control_plane = ControlPlane::builder()
        .id(CONTROL_PLANE_ID)
        .url(format!(
            "http://localhost:{}/api/v1/callback",
            cfg.signaling.port
        ))
        .build();
    seed_control_plane(&ctx, &control_planes, &control_plane).await?;

    let token_manager = Arc::new(create_token_manager(cfg, tokens).await?);
    let handler = TokenHandler::new(token_manager.clone());

    let sdk = sdk(ctx, flows, control_planes, handler).await;

    let barrier = Arc::new(Barrier::new(4));

    start_signaling(
        cfg.signaling.port,
        sdk.clone(),
        control_plane.clone(),
        barrier.clone(),
    )
    .await;

    start_public_api(
        cfg.public_api.port,
        token_manager.clone(),
        sdk.clone(),
        barrier.clone(),
    )
    .await;

    start_token_api(
        cfg.token_api.port,
        token_manager.clone(),
        sdk,
        barrier.clone(),
    )
    .await;

    tracing::info!("DataPlane is ready");
    barrier.wait().await;
    Ok(())
}

async fn sdk<C, R, CP>(
    ctx: C,
    repo: R,
    control_plane_repo: CP,
    handler: TokenHandler<C>,
) -> DataPlaneSdk<C>
where
    C: TransactionalContext + 'static,
    C::Transaction: Send,
    R: DataFlowRepo<Transaction = C::Transaction> + 'static,
    CP: ControlPlaneRepo<Transaction = C::Transaction> + 'static,
{
    DataPlaneSdk::builder(ctx)
        .with_repo(repo)
        .with_control_plane_repo(control_plane_repo)
        .with_handler(handler)
        .build()
        .unwrap()
}

/// Registers the control plane that data flows report back to, so that
/// notification callbacks can later resolve its URL by id.
async fn seed_control_plane<C, CP>(
    ctx: &C,
    control_planes: &CP,
    control_plane: &ControlPlane,
) -> anyhow::Result<()>
where
    C: TransactionalContext,
    CP: ControlPlaneRepo<Transaction = C::Transaction>,
{
    let mut tx = ctx.begin().await?;
    control_planes.create(&mut tx, control_plane).await?;
    tx.commit().await?;

    Ok(())
}

async fn create_token_manager<
    T: TransactionalContext,
    R: TokenRepo<Transaction = T::Transaction> + 'static,
>(
    cfg: &crate::config::DataPlaneConfig,
    repo: R,
) -> anyhow::Result<TokenManager<T>> {
    let public_api = cfg
        .public_api
        .api_url
        .clone()
        .unwrap_or_else(|| format!("http://localhost:{}/api/v1/public", cfg.public_api.port));

    Ok(TokenManager::builder()
        .url(public_api)
        .repo(Box::new(repo))
        .build())
}
