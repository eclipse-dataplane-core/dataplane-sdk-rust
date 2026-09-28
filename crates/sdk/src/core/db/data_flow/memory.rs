use crate::core::{
    db::memory::{MemoryRepo, MemoryTransaction},
    error::{DbError, DbResult},
    model::data_flow::DataFlow,
};

use super::DataFlowRepo;

/// In-memory [`DataFlowRepo`]. Entries are keyed by (participant context id, flow id), so flows
/// of different participant contexts never collide or see each other.
#[derive(Default, Clone)]
pub struct MemoryDataFlowRepo(MemoryRepo<DataFlow>);

/// Builds the storage key for a flow. The participant context id is length-prefixed so the
/// encoding is unambiguous whatever characters either id contains.
fn key(participant_context_id: &str, flow_id: &str) -> String {
    format!(
        "{}:{}{}",
        participant_context_id.len(),
        participant_context_id,
        flow_id
    )
}

/// Rewrites errors from the generic store so they name the flow id rather than the internal key.
fn map_err(err: DbError, flow_id: &str) -> DbError {
    match err {
        DbError::AlreadyExists(_) => {
            DbError::AlreadyExists(format!("Data flow with id {} already exists", flow_id))
        }
        DbError::NotFound(_) => {
            DbError::NotFound(format!("Data flow with id {} not found", flow_id))
        }
        other => other,
    }
}

#[async_trait::async_trait]
impl DataFlowRepo for MemoryDataFlowRepo {
    type Transaction = MemoryTransaction;
    async fn create(&self, _tx: &mut Self::Transaction, flow: &DataFlow) -> DbResult<()> {
        self.0
            .create(&key(&flow.participant_context_id, &flow.id), flow)
            .await
            .map_err(|e| map_err(e, &flow.id))
    }

    async fn fetch_by_id(
        &self,
        _tx: &mut Self::Transaction,
        participant_context_id: &str,
        flow_id: &str,
    ) -> DbResult<Option<DataFlow>> {
        self.0
            .fetch_by_id(&key(participant_context_id, flow_id))
            .await
    }

    async fn update(&self, _tx: &mut Self::Transaction, flow: &DataFlow) -> DbResult<()> {
        self.0
            .update(&key(&flow.participant_context_id, &flow.id), flow)
            .await
            .map_err(|e| map_err(e, &flow.id))
    }

    async fn delete(
        &self,
        _tx: &mut Self::Transaction,
        participant_context_id: &str,
        flow_id: &str,
    ) -> DbResult<()> {
        self.0
            .delete(&key(participant_context_id, flow_id))
            .await
            .map_err(|e| map_err(e, flow_id))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::db::memory::MemoryContext;
    use crate::core::db::memory::MemoryTransaction;
    use crate::core::db::test_suite::Tester;
    use crate::core::db::test_suite::generate_data_flow_store_tests;
    use crate::core::db::tx::TransactionalContext;

    use super::MemoryDataFlowRepo;

    pub struct MemoryTester {
        repo: MemoryDataFlowRepo,
        ctx: MemoryContext,
    }

    impl Tester<MemoryDataFlowRepo, MemoryContext> for MemoryTester {
        async fn create() -> Self {
            let ctx = MemoryContext;
            let repo = MemoryDataFlowRepo::default();
            MemoryTester { repo, ctx }
        }

        fn store(&self) -> &MemoryDataFlowRepo {
            &self.repo
        }

        async fn begin(&self) -> MemoryTransaction {
            self.ctx.begin().await.expect("Failed to begin transaction")
        }
    }

    generate_data_flow_store_tests!(MemoryTester);
}
