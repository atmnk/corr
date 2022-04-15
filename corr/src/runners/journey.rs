use corr_lib::core::runtime::Context as CorrContext;
use std::sync::Arc;
use futures::lock::Mutex;
use crate::client;
use crate::interfaces::terminal::Terminal;
pub struct JourneyRunner;
impl JourneyRunner {
    pub async fn run(target:String,journey:String){
        let jp= client::unpack(target).unwrap();
        Self::run_journey_in(jp,journey).await;
    }
    pub async fn run_journey_in(jp:String,journey:String){
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
            let context = CorrContext::new(Arc::new(Mutex::new(terminal.get_if())),jrns);
            tokio::spawn(async move {
                client::start(jn, context).await;
            });
            terminal.start().await;
        }
        // let (_,jrn) = Journey::parser(j.as_str()).unwrap();//Self::get_journey(jp,journey);


    }
}
