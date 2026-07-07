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

use crate::core::{error::DbResult, model::control_plane::ControlPlane};
pub mod memory;

#[cfg(test)]
use crate::core::db::tx::MockTransaction;

#[async_trait::async_trait]
#[cfg_attr(test, mockall::automock(type Transaction = MockTransaction;))]
pub trait ControlPlaneRepo: Send + Sync {
    type Transaction;

    async fn create(
        &self,
        tx: &mut Self::Transaction,
        control_plane: &ControlPlane,
    ) -> DbResult<()>;

    async fn fetch_by_id(
        &self,
        tx: &mut Self::Transaction,
        control_plane_id: &str,
    ) -> DbResult<Option<ControlPlane>>;
}
