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

use sqlx::{Error, types::Json};

use dataplane_sdk::core::{
    db::data_flow::DataFlowRepo,
    error::{DbError, DbResult},
    model::data_flow::DataFlow,
};

use crate::{
    PgTransaction, data_flow::model::DataFlow as DbDataFlow, data_flow::model::DataFlowState,
    data_flow::model::DataFlowType,
};

#[derive(Default)]
pub struct PgDataFlowRepo;

#[async_trait::async_trait]
impl DataFlowRepo for PgDataFlowRepo {
    type Transaction = PgTransaction;

    async fn create(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> DbResult<()> {
        let result = sqlx::query(
            r#"
            INSERT INTO data_flows (id, participant_id, dataspace_context, participant_context_id, counter_party_id, dataset_id, agreement_id, state, transfer_type, type, data_address, callback_address, labels, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(&flow.id)
        .bind(&flow.participant_id)
        .bind(&flow.dataspace_context)
        .bind(&flow.participant_context_id)
        .bind(&flow.counter_party_id)
        .bind(&flow.dataset_id)
        .bind(&flow.agreement_id)
        .bind(DataFlowState::from(flow.state.clone()))
        .bind(&flow.transfer_type)
        .bind(DataFlowType::from(flow.kind.clone()))
        .bind(flow.data_address.clone().map(Json))
        .bind(&flow.callback_address)
        .bind(Json(flow.labels.clone()))
        .bind(Json(flow.metadata.clone()))
        .bind(flow.created_at)
        .bind(flow.updated_at)
        .execute(&mut *tx.0)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(Error::Database(db)) if db.is_unique_violation() => Err(DbError::AlreadyExists(
                format!("Data flow with id {} already exists", flow.id),
            )),
            Err(err) => Err(DbError::Generic(Box::new(err))),
        }
    }

    async fn fetch_by_id(
        &self,
        tx: &mut Self::Transaction,
        flow_id: &str,
    ) -> DbResult<Option<DataFlow>> {
        Ok(sqlx::query_as::<_, DbDataFlow>(
            r#"
            SELECT * FROM data_flows where id = $1
            "#,
        )
        .bind(flow_id)
        .fetch_optional(&mut *tx.0)
        .await
        .map_err(|err| DbError::Generic(Box::new(err)))?
        .map(|flow| flow.into()))
    }

    async fn delete(&self, tx: &mut Self::Transaction, flow_id: &str) -> DbResult<()> {
        let rows = sqlx::query(
            r#"
            DELETE FROM data_flows where id = $1
            "#,
        )
        .bind(flow_id)
        .execute(&mut *tx.0)
        .await
        .map_err(|err| DbError::Generic(Box::new(err)))?
        .rows_affected();

        if rows == 0 {
            return Err(DbError::NotFound(format!(
                "Data flow with id {} not found",
                flow_id
            )));
        }

        Ok(())
    }

    async fn update(&self, tx: &mut Self::Transaction, flow: &DataFlow) -> DbResult<()> {
        let rows = sqlx::query(
            r#"
            UPDATE data_flows SET state=$1
            WHERE id = $2
            "#,
        )
        .bind(DataFlowState::from(flow.state.clone()))
        .bind(&flow.id)
        .execute(&mut *tx.0)
        .await
        .map_err(|err| DbError::Generic(Box::new(err)))?
        .rows_affected();

        if rows == 0 {
            return Err(DbError::NotFound(format!(
                "Data flow with id {} not found",
                flow.id
            )));
        }

        Ok(())
    }
}

impl PgDataFlowRepo {
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
