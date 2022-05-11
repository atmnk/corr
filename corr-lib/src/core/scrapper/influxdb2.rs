use futures_util::stream;
use influxdb2::Client;
use influxdb2::models::DataPoint;
use crate::core::scrapper::Scrapper;
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
}
impl InfluxDB2Scrapper{
    pub fn new(url:&str,token:&str,org:&str,bucket:&str)->Self{
        InfluxDB2Scrapper {
            client: Client::new(url, org,token),
            bucket:bucket.to_string()
        }
    }
}