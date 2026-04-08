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

use uuid::Uuid;

use crate::core::{
    db::tx::Transaction,
    error::DbError,
    model::{
        data_address::DataAddress,
        data_flow::{DataFlow, DataFlowState, DataFlowType},
    },
};

use super::{data_flow::DataFlowRepo, tx::TransactionalContext};

pub trait Tester<T, Tx: TransactionalContext> {
    fn create() -> impl Future<Output = Self>;
    fn store(&self) -> &T;
    fn begin(&self) -> impl Future<Output = Tx::Transaction>;
}

pub fn create_data_flow(id: &str) -> DataFlow {
    DataFlow::builder()
        .id(id.to_string())
        .data_address(
            DataAddress::builder()
                .endpoint_type("type")
                .endpoint_properties(vec![])
                .endpoint("endpoint")
                .build(),
        )
        .participant_context_id("participant_id")
        .counter_party_id("counter_party_id")
        .state(DataFlowState::Started)
        .labels(vec!["label1".to_string(), "label2".to_string()])
        .agreement_id("agreement_id")
        .metadata(
            vec![("key".to_string(), "value".into())]
                .into_iter()
                .collect(),
        )
        .dataset_id("dataset_id")
        .dataspace_context("dataspace_context")
        .participant_id("participant_id")
        .callback_address("callback_address")
        .transfer_type("transfer_type")
        .kind(DataFlowType::Provider)
        .build()
}

pub async fn create<T, Tx: TransactionalContext>(tester: impl Tester<T, Tx>)
where
    T: DataFlowRepo<Transaction = Tx::Transaction>,
{
    let store = tester.store();
    let mut tx = tester.begin().await;

    let id = Uuid::new_v4().to_string();
    let transfer = create_data_flow(&id);

    store.create(&mut tx, &transfer).await.unwrap();

    let saved = store.fetch_by_id(&mut tx, &id).await.unwrap().unwrap();
    assert_eq!(saved, transfer);
}

pub async fn create_duplicate<T, Tx: TransactionalContext>(tester: impl Tester<T, Tx>)
where
    T: DataFlowRepo<Transaction = Tx::Transaction>,
{
    let store = tester.store();
    let mut tx = tester.begin().await;

    let id = Uuid::new_v4().to_string();
    let transfer = create_data_flow(&id);

    store.create(&mut tx, &transfer).await.unwrap();
    let result = store.create(&mut tx, &transfer).await;

    assert!(matches!(result, Err(DbError::AlreadyExists(..))));
}

pub async fn create_rollback<T, Tx: TransactionalContext>(tester: impl Tester<T, Tx>)
where
    T: DataFlowRepo<Transaction = Tx::Transaction>,
{
    let store = tester.store();
    let mut tx = tester.begin().await;

    let id = Uuid::new_v4().to_string();
    let transfer = create_data_flow(&id);

    store.create(&mut tx, &transfer).await.unwrap();

    let saved = store.fetch_by_id(&mut tx, &id).await.unwrap().unwrap();
    assert_eq!(saved, transfer);

    tx.rollback().await.expect("Failed to rollback transaction");

    let mut tx = tester.begin().await;
    let result = store.fetch_by_id(&mut tx, &id).await.unwrap();

    assert!(result.is_none());
}

pub async fn delete<T, Tx: TransactionalContext>(tester: impl Tester<T, Tx>)
where
    T: DataFlowRepo<Transaction = Tx::Transaction>,
{
    let store = tester.store();
    let mut tx = tester.begin().await;

    let transfer = create_data_flow(&Uuid::new_v4().to_string());
    let transfer_2 = create_data_flow(&Uuid::new_v4().to_string());

    store.create(&mut tx, &transfer).await.unwrap();
    store.create(&mut tx, &transfer_2).await.unwrap();

    store.delete(&mut tx, &transfer_2.id).await.unwrap();

    let result = store.fetch_by_id(&mut tx, &transfer_2.id).await.unwrap();
    assert!(result.is_none());

    let result = store.fetch_by_id(&mut tx, &transfer.id).await.unwrap();
    assert!(result.is_some());
}

pub async fn delete_not_found<T, Tx: TransactionalContext>(tester: impl Tester<T, Tx>)
where
    T: DataFlowRepo<Transaction = Tx::Transaction>,
{
    let store = tester.store();
    let mut tx = tester.begin().await;

    let id = Uuid::new_v4().to_string();

    let result = store.delete(&mut tx, &id).await;

    assert!(matches!(result, Err(DbError::NotFound(..))));
}

pub async fn update<T, Tx: TransactionalContext>(tester: impl Tester<T, Tx>)
where
    T: DataFlowRepo<Transaction = Tx::Transaction>,
{
    let store = tester.store();
    let mut tx = tester.begin().await;

    let transfer = create_data_flow("1");
    let mut updated = transfer.clone();

    updated.state = DataFlowState::Suspended;

    store.create(&mut tx, &transfer).await.unwrap();

    store.update(&mut tx, &updated).await.unwrap();

    let updated = store
        .fetch_by_id(&mut tx, &transfer.id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.state, DataFlowState::Suspended);

    let result = store
        .fetch_by_id(&mut tx, &transfer.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(result, updated);
}

pub async fn update_not_found<T, Tx: TransactionalContext>(tester: impl Tester<T, Tx>)
where
    T: DataFlowRepo<Transaction = Tx::Transaction>,
{
    let store = tester.store();
    let mut tx = tester.begin().await;

    let id = Uuid::new_v4().to_string();
    let transfer = create_data_flow(&id);
    let result = store.update(&mut tx, &transfer).await;

    assert!(matches!(result, Err(DbError::NotFound(..))));
}

#[macro_export]
macro_rules! generate_data_flow_store_tests {
    ($tester:ident) => {
        macro_rules! test {
            ($title: ident, $func: path) => {
                $crate::core::db::test_suite::declare_test_fn!($tester, $title, $func);
            };
        }

        test!(data_flow_create, $crate::core::db::test_suite::create);
        test!(
            data_flow_create_duplicate,
            $crate::core::db::test_suite::create_duplicate
        );
        test!(data_flow_delete, $crate::core::db::test_suite::delete);
        test!(
            data_flow_delete_not_found,
            $crate::core::db::test_suite::delete_not_found
        );
        test!(data_flow_update, $crate::core::db::test_suite::update);
        test!(
            data_flow_update_not_found,
            $crate::core::db::test_suite::update_not_found
        );
    };
}

#[macro_export]
macro_rules! declare_test_fn {
    ($storage: ident, $title: ident, $func: path) => {
        #[tokio::test]
        async fn $title() {
            let storage = $storage::create().await;

            $func(storage).await;
        }
    };
}
pub use declare_test_fn;
pub use generate_data_flow_store_tests;
