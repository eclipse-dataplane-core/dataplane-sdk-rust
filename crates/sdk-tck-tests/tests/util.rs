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

use std::{
    net::{SocketAddr, TcpStream},
    path::{self, Path},
    str::FromStr,
    sync::{Arc, LazyLock, Mutex},
};

use axum::{Extension, Router};
use dataplane_sdk::{
    core::{
        db::{
            data_flow::DataFlowRepo,
            tx::{Transaction, TransactionalContext},
        },
        handler::DataFlowHandler,
        model::participant::ParticipantContext,
    },
    sdk::DataPlaneSdk,
};
use dataplane_sdk_axum::router::router;
use dataplane_sdk_postgres::{PgContext, PgDataFlowRepo};
use futures::FutureExt;
use futures::future::BoxFuture;
use regex::Regex;
use testcontainers::{
    GenericImage, ImageExt,
    core::{ContainerPort, Host, IntoContainerPort, Mount, WaitFor, logs::consumer::LogConsumer},
    runners::AsyncRunner,
};
use testcontainers_modules::postgres::Postgres;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::{Level, info};

pub async fn setup_postgres_container() -> (
    PgContext,
    PgDataFlowRepo,
    testcontainers::ContainerAsync<Postgres>,
) {
    let container = Postgres::default().start().await.unwrap();

    let connection_string = format!(
        "postgresql://postgres:postgres@localhost:{}/postgres",
        container.get_host_port_ipv4(5432).await.unwrap()
    );

    // Wait for PostgreSQL to be ready with timeout
    let ctx = tokio::time::timeout(tokio::time::Duration::from_secs(5), async {
        loop {
            match setup_pg(&connection_string).await {
                Ok(pool) => break pool,
                Err(_) => tokio::task::yield_now().await,
            }
        }
    })
    .await
    .unwrap_or_else(|_| panic!("PostgreSQL launch failed"));

    (ctx.0, ctx.1, container)
}

async fn setup_pg(url: &str) -> anyhow::Result<(PgContext, PgDataFlowRepo)> {
    let ctx = PgContext::connect(url).await?;

    let mut tx = ctx.begin().await?;
    let repo = PgDataFlowRepo;

    repo.migrate(&mut tx).await?;

    tx.commit().await?;

    Ok((ctx, repo))
}

pub async fn setup_tck_container(
    reporter: TckTestReporter,
) -> testcontainers::ContainerAsync<GenericImage> {
    let path = Path::new("tests/dps.tck.properties");
    GenericImage::new("eclipsedataspacetck/dps-tck-runtime", "1.1.2")
        .with_exposed_port(8083.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Test run complete"))
        .with_mapped_port(8083, ContainerPort::Tcp(8083))
        .with_mount(Mount::bind_mount(
            path::absolute(path).unwrap().as_os_str().to_str().unwrap(),
            "/etc/tck/config.properties",
        ))
        .with_host("host.docker.internal", Host::HostGateway)
        .with_log_consumer(reporter)
        .start()
        .await
        .unwrap()
}

pub async fn start_signaling<C>(port: u16, sdk: DataPlaneSdk<C>)
where
    C: TransactionalContext + 'static,
    C::Transaction: Send,
{
    let addr = SocketAddr::from_str(&format!("0.0.0.0:{port}")).expect("Invalid socket address");

    let p_context = ParticipantContext::builder().id("tck-participant").build();
    let router = router().layer(Extension(p_context));

    launch_server("Signaling API", router, sdk.clone(), addr).await;
}

pub async fn launch_server<S: Clone + Send + Sync + 'static>(
    tag: &str,
    router: Router<S>,
    state: S,
    addr: std::net::SocketAddr,
) {
    let service_name = tag.to_string();
    tokio::task::spawn(async move {
        let app = router
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(DefaultOnResponse::new().level(Level::INFO)),
            )
            .with_state(state);

        if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
            tracing::debug!(
                "{service_name} listening on {}",
                listener.local_addr().expect("failed to get local address")
            );
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await;

            tracing::debug!("{service_name} server has been shut down");
        } else {
            tracing::error!("{service_name} failed to bind to {addr}");
        }
    });

    wait_for_server(addr).await;
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

pub async fn wait_for_server(socket: std::net::SocketAddr) {
    for _ in 0..10 {
        if TcpStream::connect_timeout(&socket, std::time::Duration::from_millis(25)).is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
}

pub async fn sdk<C, R, H>(ctx: C, repo: R, handler: H) -> DataPlaneSdk<C>
where
    C: TransactionalContext + 'static,
    C::Transaction: Send,
    R: DataFlowRepo<Transaction = C::Transaction> + 'static,
    H: DataFlowHandler<Transaction = C::Transaction> + 'static,
{
    DataPlaneSdk::builder(ctx)
        .with_repo(repo)
        .with_handler(handler)
        .build()
        .unwrap()
}

#[derive(Clone, Default)]
pub struct TckTestReporter {
    failure: Arc<Mutex<Vec<String>>>,
}

static FAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"FAILED: (\w+:\d+-\d+)").unwrap());

impl LogConsumer for TckTestReporter {
    fn accept<'a>(&'a self, record: &'a testcontainers::core::logs::LogFrame) -> BoxFuture<'a, ()> {
        let log = String::from_utf8_lossy(record.bytes());

        if let Some(caps) = FAIL_REGEX.captures(&log) {
            self.failure.lock().unwrap().push(caps[1].to_string());
        }

        info!("{}", &log[..log.len() - 1]);

        futures::future::ready(()).boxed()
    }
}

impl TckTestReporter {
    pub fn failures(&self) -> Vec<String> {
        self.failure.lock().unwrap().clone()
    }
}
