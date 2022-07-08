use std::env;
use corr_lib::core::runtime::Context as CorrContext;
use std::sync::Arc;
use futures::lock::Mutex;
use corr_lib::core::scrapper::influxdb2::InfluxDB2Scrapper;
use corr_lib::core::scrapper::none::NoneScraper;
use corr_lib::core::scrapper::Scrapper;
use crate::{client, Out};
use crate::interfaces::terminal::Terminal;
pub struct JourneyRunner;
impl JourneyRunner {
    pub async fn run(target:String,journey:String,out:Out,debug:bool){
        let jp= client::unpack(target).unwrap();
        Self::run_journey_in(jp,journey,out,debug).await;
    }
    pub async fn run_journey_in(jp:String,journey:String,out:Out,debug:bool){
        let jrns = client::get_journeis_in(format!("{}/src", jp)).await.unwrap();
        let j = if journey.clone().eq("<default>"){
            jrns.get(0).map(|j|j.clone())
        } else {
            let mut jn = Option::None;
            for jrn in &jrns {
                if jrn.name.eq(&journey.clone()) {
                    jn = Option::Some(jrn.clone());
                    break;
                }
            }
            jn
        };
        if let Some(jn) = j {
            let mut terminal = Terminal::new();
            let scrapper:Box<dyn Scrapper> = match out {
                Out::InfluxDB2=>{
                    Box::new(InfluxDB2Scrapper::new(env::var("J_INFLUX_URL").unwrap().as_str(),env::var("J_INFLUX_TOKEN").unwrap().as_str(),env::var("J_INFLUX_ORG").unwrap().as_str(),env::var("J_INFLUX_BUCKET").unwrap().as_str()))
                },
                _=> Box::new(NoneScraper{})
            };
            let context = CorrContext::new(Arc::new(Mutex::new(terminal.get_if())),jrns,Arc::new(scrapper),debug);
            tokio::spawn(async move {
                client::start(jn, context).await;
            });
            terminal.start().await;
        }
        // let (_,jrn) = Journey::parser(j.as_str()).unwrap();//Self::get_journey(jp,journey);


    }
}
