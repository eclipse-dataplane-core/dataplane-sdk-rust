use bon::Builder;
use dataplane_sdk::core::{
    db::tx::TransactionalContext,
    model::data_address::{DataAddress, EndpointProperty},
};
use uuid::Uuid;

use crate::tokens::repo::TokenRepo;

use super::model::TokenError;

#[derive(Builder)]
pub struct TokenManager<T: TransactionalContext> {
    #[builder(into)]
    pub(crate) url: String,
    repo: Box<dyn TokenRepo<Transaction = T::Transaction> + Send + Sync>,
}

impl<T: TransactionalContext> TokenManager<T> {
    pub async fn create_token(&self) -> Result<(Uuid, String, DataAddress), TokenError> {
        let token_id = Uuid::new_v4();

        let data_address = DataAddress::builder()
            .endpoint_type("HTTP")
            .endpoint_properties(self.endpoint_properties(token_id)?)
            .endpoint(&self.url)
            .build();

        Ok((token_id, self.url.clone(), data_address))
    }

    fn endpoint_properties(&self, token_id: Uuid) -> Result<Vec<EndpointProperty>, TokenError> {
        Ok(vec![
            EndpointProperty::builder()
                .name("endpoint")
                .value(self.url.clone())
                .build(),
            EndpointProperty::builder()
                .name("access_token")
                .value(token_id)
                .build(),
            EndpointProperty::builder()
                .name("token_type")
                .value("Bearer")
                .build(),
        ])
    }

    pub fn repo(&self) -> &(dyn TokenRepo<Transaction = T::Transaction> + Send + Sync) {
        self.repo.as_ref()
    }
}
