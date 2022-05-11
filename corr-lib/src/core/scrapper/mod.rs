pub mod influxdb2;
pub mod none;
use async_trait::async_trait;
#[async_trait]
pub trait Scrapper:Send+Sync{
    async fn ingest(&self,series:&str,data:f64,tags:Vec<(String,String)>);
}