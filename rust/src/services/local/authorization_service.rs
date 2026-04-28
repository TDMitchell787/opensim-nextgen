use anyhow::Result;
use async_trait::async_trait;
use tracing::debug;
use uuid::Uuid;

use crate::services::traits::AuthorizationServiceTrait;

pub struct LocalAuthorizationService;

impl LocalAuthorizationService {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AuthorizationServiceTrait for LocalAuthorizationService {
    async fn is_authorized_for_region(
        &self,
        user_id: Uuid,
        _first_name: &str,
        _last_name: &str,
        region_id: Uuid,
    ) -> Result<(bool, String)> {
        debug!(
            "[AUTHORIZATION] Checking user {} for region {}",
            user_id, region_id
        );
        Ok((true, String::new()))
    }
}
