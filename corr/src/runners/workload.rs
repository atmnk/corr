use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use futures::lock::Mutex;
use tokio::time::sleep;
use crate::client;
use crate::interfaces::standalone::StandAloneInterface;
use corr_lib::core::runtime::Context as CorrContext;
use corr_lib::core::Value;

pub struct WorkLoadRunner;
impl WorkLoadRunner{
    pub async fn run(target:String,workload:String){
        let jp= client::unpack(target).unwrap();
        Self::run_workload_in(jp, workload).await;
    }
    pub async fn run_workload_in(jp:String, workload:String){
        let wrklds = client::get_workloads_in(format!("{}/src", jp)).await.unwrap();
        let jrns = client::get_journeis_in(format!("{}/src", jp)).await.unwrap();
        let j = if workload.clone().eq("<default>"){
            wrklds.get(0).map(|j|j.clone())
        } else {
            let mut jn = Option::None;
            for jrn in &wrklds {
                if jrn.name.eq(&workload.clone()) {
                    jn = Option::Some(jrn.clone());
                    break;
                }
            }
            jn
        };
        if let Some(wl) = j {
            let context = CorrContext::new(Arc::new(Mutex::new(StandAloneInterface{})), jrns.clone());
            let mut jn = Option::None;
            for jrn in &jrns {
                if jrn.name.eq(&wl.journey) {
                    jn = Option::Some(jrn.clone());
                    break;
                }
            }
            let mut handles = vec![];
            for concurrentUser in 0..wl.concurrentUsers {
                let new_jn = jn.clone();
                let new_ct = CorrContext::from(&context).await;
                let d = wl.duration.clone();
                let t=tokio::spawn(async move {
                    let start = SystemTime::now();
                    let since_the_epoch = start
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards");
                    let in_ms = since_the_epoch.as_millis();
                    let mut elapsed:u128 = 0;
                    let mut iter = 1;

                    while elapsed<(d * 1000) {
                        let new_ct = CorrContext::from(&new_ct).await;
                        let new_jn = new_jn.clone();
                        new_ct.define("VU".to_string(),Value::PositiveInteger(concurrentUser as u128)).await;
                        new_ct.define("ITER".to_string(),Value::PositiveInteger(iter)).await;
                        client::start(new_jn.unwrap(), new_ct).await;
                        let start = SystemTime::now();
                        let since_the_epoch = start
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards");
                        let in_ms_now = since_the_epoch.as_millis();
                        elapsed = in_ms_now - in_ms;
                        iter = iter + 1
                    }

                });
                sleep(Duration::from_millis(wl.perUserRampUp)).await;
                handles.push(t);
            }
            futures::future::join_all(handles).await;
        }
    }
}