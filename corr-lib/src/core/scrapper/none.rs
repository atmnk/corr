use async_trait::async_trait;
use crate::core::scrapper::Scrapper;

pub struct NoneScraper;
#[async_trait]
impl Scrapper for NoneScraper{
    async fn ingest(&self, series: &str, data: f64, tags: Vec<(String, String)>) {

    }
}