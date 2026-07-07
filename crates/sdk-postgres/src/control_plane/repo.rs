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

use sqlx::Error;

use dataplane_sdk::core::{
    db::control_plane::ControlPlaneRepo,
    error::{DbError, DbResult},
    model::control_plane::ControlPlane,
};

use crate::{PgTransaction, control_plane::model::ControlPlane as DbControlPlane};

#[derive(Default)]
pub struct PgControlPlaneRepo;

#[async_trait::async_trait]
impl ControlPlaneRepo for PgControlPlaneRepo {
    type Transaction = PgTransaction;

    async fn create(
        &self,
        tx: &mut Self::Transaction,
        control_plane: &ControlPlane,
    ) -> DbResult<()> {
        let result = sqlx::query(
            r#"
            INSERT INTO control_planes (id, url)
            VALUES ($1, $2)
            "#,
        )
        .bind(&control_plane.id)
        .bind(&control_plane.url)
        .execute(&mut *tx.0)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(Error::Database(db)) if db.is_unique_violation() => Err(DbError::AlreadyExists(
                format!("Control plane with id {} already exists", control_plane.id),
            )),
            Err(err) => Err(DbError::Generic(Box::new(err))),
        }
    }

    async fn fetch_by_id(
        &self,
        tx: &mut Self::Transaction,
        control_plane_id: &str,
    ) -> DbResult<Option<ControlPlane>> {
        Ok(sqlx::query_as::<_, DbControlPlane>(
            r#"
            SELECT * FROM control_planes where id = $1
            "#,
        )
        .bind(control_plane_id)
        .fetch_optional(&mut *tx.0)
        .await
        .map_err(|err| DbError::Generic(Box::new(err)))?
        .map(|control_plane| control_plane.into()))
    }
}

impl PgControlPlaneRepo {
    pub async fn migrate(&self, tx: &mut PgTransaction) -> DbResult<()> {
        let mut migrator = sqlx::migrate!("./migrations");
        migrator.set_ignore_missing(true);

        migrator
            .run(&mut *tx.0)
            .await
            .map_err(|err| DbError::Generic(Box::new(err)))?;

        Ok(())
    }
}
