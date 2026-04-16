use crate::error::Result;
use crate::models::announcement::Announcements;
use crate::rest::client::LighterRestClient;

impl LighterRestClient {
    pub async fn get_announcements(&self) -> Result<Announcements> {
        self.get("/api/v1/announcement").await
    }
}
