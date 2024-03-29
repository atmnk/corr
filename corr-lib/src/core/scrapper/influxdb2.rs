
use std::sync::{Arc};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures_util::stream;
use influxdb2::Client;
use influxdb2::models::DataPoint;
use crate::core::scrapper::{Metrics, Scrapper};
use async_trait::async_trait;

use tokio::sync::RwLock;
use tokio::time::sleep;
#[derive(Clone)]
pub struct MDP{
    series:String,
    data:f64,
    tags:Vec<(String,String)>,
    tt:i64,
}
pub struct InfluxDB2Scrapper{
    client:Arc<Client>,
    bucket:String,
    data_points:RwLock<Vec<MDP>>,
}
#[async_trait]
impl Scrapper for InfluxDB2Scrapper{
    async fn start_metrics_loop(&self) {
        loop {
            let points_copy;
            {
                let mut points = self.data_points.write().await;
                points_copy = (*points).clone();
                *points = vec![];
            }
            let pts :Vec<DataPoint> = points_copy.iter().map(|p| {
                let mut builder = DataPoint::builder(p.series.as_str());
                for (tag_name,tag_value) in &p.tags{
                    builder=builder.tag(tag_name.as_str(),tag_value.as_str());
                }
                builder = builder.field("value",p.data);
                builder = builder.timestamp(p.tt);
                builder.build().unwrap()
            }).collect();
            let cl = self.client.clone();
            let b = self.bucket.clone();
            let task = async move ||{
                cl.write(b.as_str(),stream::iter(pts)).await.unwrap();
            };
            tokio::spawn( task());
            sleep(Duration::from_millis(500)).await;
        }
    }

    async fn ingest(&self,series:&str,data:f64,tags:Vec<(String,String)>) {

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH).unwrap();
        let dpv = MDP{
            series:series.to_string(),
            data,
            tags,
            tt:since_the_epoch.as_nanos() as i64
        };{
            let mut dp = self.data_points.write().await;
            (*dp).push(dpv);
        }

   }
    async fn ingest_metric(&self, metrics: Arc<Metrics>, tag: (String, String)) {
        let mut iters = metrics.iterations.write().await;
        let i = *iters;
        *iters = 0.0;
        let mut errors = metrics.errors.write().await;
        let e = *errors;
        *errors = 0.0;
        let mut builder_iterations = DataPoint::builder("iteration_count");
        builder_iterations =builder_iterations.tag(tag.0.clone(),tag.1.clone());
        builder_iterations = builder_iterations.field("value", i);
        let mut builder_errors = DataPoint::builder("errors");
        builder_errors = builder_errors.tag(tag.0.clone(),tag.1.clone());
        builder_errors = builder_errors.field("value",e );
        let _ = self.client.write(self.bucket.as_str(),stream::iter(vec![builder_iterations.build().unwrap(),builder_errors.build().unwrap()])).await;
    }
}
impl InfluxDB2Scrapper{
    pub fn new(url:&str,token:&str,org:&str,bucket:&str)->Self{
        InfluxDB2Scrapper {
            data_points:RwLock::new(vec![]),
            client: Arc::new(Client::new(url, org,token)),
            bucket:bucket.to_string()
        }
    }
}