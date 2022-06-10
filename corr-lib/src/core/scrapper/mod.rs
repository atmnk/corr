pub mod influxdb2;
pub mod none;

use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub struct Metrics {
    pub vus: RwLock<f64>,
    pub iterations: RwLock<f64>,
    pub iteration_duration: RwLock<f64>,
    pub errors: RwLock<f64>
}
impl Metrics{
    pub fn new()->Self{
        Metrics{
            vus:RwLock::new(0.0),
            iterations:RwLock::new(0.0),
            iteration_duration:RwLock::new(0.0),
            errors:RwLock::new(0.0)
        }
    }
}

#[async_trait]
pub trait Scrapper:Send+Sync{
    async fn start_metrics_loop(&self);
    async fn ingest(&self,series:&str,data:f64,tags:Vec<(String,String)>);
    async fn ingest_metric(&self,metrics:Arc<Metrics>,tag:(String,String));
}