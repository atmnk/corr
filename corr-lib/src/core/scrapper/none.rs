use std::sync::Arc;
use async_trait::async_trait;
use crate::core::scrapper::{Metrics, Scrapper};

pub struct NoneScraper;
#[async_trait]
impl Scrapper for NoneScraper{
    async fn start_metrics_loop(&self) {

    }

    async fn ingest(&self, series: &str, data: f64, tags: Vec<(String, String)>) {

    }


    async fn ingest_metric(&self, metrics: Arc<Metrics>, tag: (String, String)) {

    }
}