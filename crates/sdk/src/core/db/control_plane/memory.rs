use crate::core::{
    db::memory::{MemoryRepo, MemoryTransaction},
    error::DbResult,
    model::control_plane::ControlPlane,
};

use super::ControlPlaneRepo;

#[derive(Default, Clone)]
pub struct MemoryControlPlaneRepo(MemoryRepo<ControlPlane>);

#[async_trait::async_trait]
impl ControlPlaneRepo for MemoryControlPlaneRepo {
    type Transaction = MemoryTransaction;

    async fn create(
        &self,
        _tx: &mut Self::Transaction,
        control_plane: &ControlPlane,
    ) -> DbResult<()> {
        self.0.create(&control_plane.id, control_plane).await
    }

    async fn fetch_by_id(
        &self,
        _tx: &mut Self::Transaction,
        control_plane_id: &str,
    ) -> DbResult<Option<ControlPlane>> {
        self.0.fetch_by_id(control_plane_id).await
    }
}
