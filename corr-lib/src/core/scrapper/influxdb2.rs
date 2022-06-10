use std::sync::Arc;
use test::test::Metric;
use futures_util::stream;
use influxdb2::Client;
use influxdb2::models::DataPoint;
use crate::core::scrapper::{Metrics, Scrapper};
use async_trait::async_trait;
pub struct InfluxDB2Scrapper{
    client:Client,
    bucket:String,
}
#[async_trait]
impl Scrapper for InfluxDB2Scrapper{
    async fn ingest(&self,series:&str,data:f64,tags:Vec<(String,String)>) {
        let mut builder = DataPoint::builder(series);
        for (tag_name,tag_value) in tags{
            builder=builder.tag(tag_name.as_str(),tag_value.as_str());
        }
        builder = builder.field("value",data);
        self.client.write(self.bucket.as_str(),stream::iter(vec![builder.build().unwrap()])).await;
    }

    async fn ingest_metric(&self, metrics: Arc<Metrics>, tag: (String, String)) {
        let mut iters = metrics.iterations.write().await;
        let i = *iters;
        *iters = 0.0;
        let mut errors = metrics.errors.write().await;
        let e = *errors;
        *errors = 0.0;
        let mut iterd = metrics.iteration_duration.write().await;
        let id = *iterd;
        *iterd = 0.0;
        let mut builder_vu = DataPoint::builder("vus");
        builder_vu = builder_vu.tag(tag.0.clone(),tag.1.clone());
        builder_vu = builder_vu.field("value",*(metrics.vus.read().await) );
        let mut builder_iterations = DataPoint::builder("iteration_count");
        builder_iterations =builder_iterations.tag(tag.0.clone(),tag.1.clone());
        builder_iterations = builder_iterations.field("value", i);
        let mut builder_errors = DataPoint::builder("errors");
        builder_errors = builder_errors.tag(tag.0.clone(),tag.1.clone());
        builder_errors = builder_errors.field("value",e );
        if (i>0.0){
            let mut builder_id = DataPoint::builder("iteration_duration");
            builder_id = builder_id.tag(tag.0.clone(),tag.1.clone());
            builder_id = builder_id.field("value",id / i );
            self.client.write(self.bucket.as_str(),stream::iter(vec![builder_vu.build().unwrap(),builder_iterations.build().unwrap(),builder_errors.build().unwrap(),builder_id.build().unwrap()])).await;
        } else {
            self.client.write(self.bucket.as_str(),stream::iter(vec![builder_vu.build().unwrap(),builder_iterations.build().unwrap(),builder_errors.build().unwrap()])).await;
        }
    }
}
impl InfluxDB2Scrapper{
    pub fn new(url:&str,token:&str,org:&str,bucket:&str)->Self{
        InfluxDB2Scrapper {
            client: Client::new(url, org,token),
            bucket:bucket.to_string()
        }
    }
}