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

use std::{ops::Deref, sync::Arc};

use crate::core::{
    db::{data_flow::DataFlowRepo, tx::TransactionalContext},
    handler::DataFlowHandler,
};

pub mod internal;

pub struct DataPlaneSdk<C: TransactionalContext>(Arc<internal::DataPlaneSdkInternal<C>>);

impl<C> Clone for DataPlaneSdk<C>
where
    C: TransactionalContext,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<C> DataPlaneSdk<C>
where
    C: TransactionalContext,
{
    pub fn builder(ctx: C) -> DataPlaneSdkBuilder<C> {
        DataPlaneSdkBuilder::new(ctx)
    }

    pub(crate) fn new(
        ctx: C,
        repo: Box<dyn DataFlowRepo<Transaction = C::Transaction>>,
        handler: Box<dyn DataFlowHandler<Transaction = C::Transaction>>,
        client: reqwest::Client,
    ) -> Self {
        Self(Arc::new(internal::DataPlaneSdkInternal {
            ctx,
            repo,
            handler,
            client,
        }))
    }
}

impl<C> Deref for DataPlaneSdk<C>
where
    C: TransactionalContext,
{
    type Target = internal::DataPlaneSdkInternal<C>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct DataPlaneSdkBuilder<C>
where
    C: TransactionalContext,
{
    ctx: C,
    repo: Option<Box<dyn DataFlowRepo<Transaction = C::Transaction>>>,
    handler: Option<Box<dyn DataFlowHandler<Transaction = C::Transaction>>>,
    client: Option<reqwest::Client>,
}

impl<C> DataPlaneSdkBuilder<C>
where
    C: TransactionalContext,
{
    pub(crate) fn new(ctx: C) -> Self {
        Self {
            ctx,
            repo: None,
            handler: None,
            client: None,
        }
    }

    pub fn with_repo(
        mut self,
        repo: impl DataFlowRepo<Transaction = C::Transaction> + 'static,
    ) -> Self {
        self.repo = Some(Box::new(repo));
        self
    }

    pub fn with_handler(
        mut self,
        handler: impl DataFlowHandler<Transaction = C::Transaction> + 'static,
    ) -> Self {
        self.handler = Some(Box::new(handler));
        self
    }

    /// Overrides the `reqwest::Client` used to send control plane notifications.
    /// Defaults to `reqwest::Client::new()` when not set.
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> Result<DataPlaneSdk<C>, String> {
        let repo = self.repo.ok_or("DataFlowRepo is not set")?;

        let handler = self.handler.ok_or("DataFlowHandler is not set")?;

        let client = self.client.unwrap_or_default();

        Ok(DataPlaneSdk::new(self.ctx, repo, handler, client))
    }
}
