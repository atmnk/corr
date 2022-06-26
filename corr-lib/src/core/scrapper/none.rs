use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tokio::time::sleep;
use crate::core::scrapper::{Metrics, Scrapper};

pub struct NoneScraper;
#[async_trait]
impl Scrapper for NoneScraper{
    async fn start_metrics_loop(&self) {
        while true {
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn ingest(&self, _series: &str, _data: f64, _tags: Vec<(String, String)>) {

    }


    async fn ingest_metric(&self, _metrics: Arc<Metrics>, _tag: (String, String)) {

    }
}