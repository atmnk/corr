use std::env;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use futures::lock::Mutex;
use core::option::Option;
use futures::stream;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep};
use crate::{client, Out};
use crate::interfaces::standalone::StandAloneInterface;
use corr_lib::core::runtime::{Context as CorrContext, Context};
use corr_lib::core::scrapper::influxdb2::InfluxDB2Scrapper;
use corr_lib::core::scrapper::none::NoneScraper;
use corr_lib::core::scrapper::Scrapper;
use corr_lib::core::Value;
use corr_lib::journey::{Executable, Journey};
use corr_lib::workload::{ModelScenario, Scenario, WorkLoad};
pub struct WorkLoadRunner;
impl WorkLoadRunner{
    pub async fn run(target:String,workload:String,out:Out){
        let jp= client::unpack(target).unwrap();
        Self::run_workload_in(jp, workload,out).await;
    }
    pub async fn run_workload_in(jp:String, workload:String,out:Out){
        let wrklds = client::get_workloads_in(format!("{}/src", jp)).await.unwrap();
        let jrns = client::get_journeis_in(format!("{}/src", jp)).await.unwrap();
        let workload = if workload.clone().eq("<default>"){
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
        if let Some(wl) = workload {
                    let scrp:Box<dyn Scrapper> = match out {
                        Out::InfluxDB2=>{
                            Box::new(InfluxDB2Scrapper::new(env::var("J_INFLUX_URL").unwrap().as_str(),env::var("J_INFLUX_TOKEN").unwrap().as_str(),env::var("J_INFLUX_ORG").unwrap().as_str(),env::var("J_INFLUX_BUCKET").unwrap().as_str()))
                        },
                        _=> Box::new(NoneScraper{})
                    };
            schedule_workload(wl,jrns,Arc::new(scrp)).await
        }
    }
    // pub async fn run_workload_in(jp:String, workload:String,out:Out){
    //     let wrklds = client::get_workloads_in(format!("{}/src", jp)).await.unwrap();
    //     let jrns = client::get_journeis_in(format!("{}/src", jp)).await.unwrap();
    //     let j = if workload.clone().eq("<default>"){
    //         wrklds.get(0).map(|j|j.clone())
    //     } else {
    //         let mut jn = Option::None;
    //         for jrn in &wrklds {
    //             if jrn.name.eq(&workload.clone()) {
    //                 jn = Option::Some(jrn.clone());
    //                 break;
    //             }
    //         }
    //         jn
    //     };
    //     if let Some(wl) = j {
    //         let scrapper:Box<dyn Scrapper> = match out {
    //             Out::InfluxDB2=>{
    //                 Box::new(InfluxDB2Scrapper::new(env::var("J_INFLUX_URL").unwrap().as_str(),env::var("J_INFLUX_TOKEN").unwrap().as_str(),env::var("J_INFLUX_ORG").unwrap().as_str(),env::var("J_INFLUX_BUCKET").unwrap().as_str()))
    //             },
    //             _=> Box::new(NoneScraper{})
    //         };
    //         let context = CorrContext::new(Arc::new(Mutex::new(StandAloneInterface{})), jrns.clone(),Arc::new(scrapper));
    //         let mut jn = Option::None;
    //         if let Some(startup_journey) = &wl.startup_journey{
    //             for jrn in &jrns {
    //                 if jrn.name.eq(startup_journey) {
    //                     jn = Option::Some(jrn.clone());
    //                     break;
    //                 }
    //             }
    //         }
    //         if let Some(jr)= &jn {
    //             jr.execute(&context).await;
    //         }
    //         for jrn in &jrns {
    //             if jrn.name.eq(&wl.journey) {
    //                 jn = Option::Some(jrn.clone());
    //                 break;
    //             }
    //         }
    //         let mut handles = vec![];
    //         let mut vus:f64 = 1.0;
    //         for concurrentUser in 0..wl.concurrentUsers {
    //             let new_jn = jn.clone();
    //             let new_ct_o = CorrContext::from(&context).await;
    //             let scrapper = new_ct_o.scrapper.clone();
    //             let d = wl.duration.clone();
    //             let t=tokio::spawn(async move {
    //                 let start = SystemTime::now();
    //                 let since_the_epoch = start
    //                     .duration_since(UNIX_EPOCH)
    //                     .expect("Time went backwards");
    //                 let in_ms = since_the_epoch.as_millis();
    //                 let mut elapsed:u128 = 0;
    //                 let mut iter = 1;
    //
    //                 while elapsed<(d * 1000) {
    //                     let new_ct = CorrContext::from(&new_ct_o).await;
    //                     let new_jn = new_jn.clone();
    //                     let name = new_jn.clone().unwrap().name;
    //                     new_ct.define("VU".to_string(),Value::PositiveInteger(concurrentUser as u128)).await;
    //                     new_ct.define("ITER".to_string(),Value::PositiveInteger(iter)).await;
    //                     client::start(new_jn.unwrap(), Context::from_without_fallback(&new_ct).await).await;
    //                     let start = SystemTime::now();
    //                     let since_the_epoch = start
    //                         .duration_since(UNIX_EPOCH)
    //                         .expect("Time went backwards");
    //                     let in_ms_now = since_the_epoch.as_millis();
    //                     elapsed = in_ms_now - in_ms;
    //                     iter = iter + 1;
    //                     new_ct.scrapper.ingest("iterations",1.0,vec![("journey".to_string(),name)]).await;
    //                 }
    //
    //             });
    //             scrapper.ingest("vus",vus,vec![("journey".to_string(),wl.journey.clone())]).await;
    //             vus=vus+1.0;
    //             sleep(Duration::from_millis(wl.perUserRampUp)).await;
    //             handles.push(t);
    //         }
    //         let el = async {
    //             loop {
    //                 context.scrapper.ingest("vus",vus,vec![("journey".to_string(),wl.journey.clone())]).await;
    //                 sleep(Duration::from_millis(500)).await;
    //             }
    //         };
    //         tokio::select! {
    //             _= el=>{},
    //             _=futures::future::join_all(handles)=>{}
    //         }
    //         context.rest_stats_store.print_stats_summary().await;
    //         context.tr_stats_store.print_stats_summary().await;
    //     }
    // }
}
pub async fn schedule_workload(workload:WorkLoad,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>){
    let joins:Vec<_> = workload.scenarios.iter().map(|sc|sc.clone()).map(|sc|schedule_scenario(sc,journeys.clone(),scrapper.clone())).collect();
    futures::future::join_all(joins).await;
}
async fn schedule_scenario(scenario:Scenario,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>){
    match scenario {
        Scenario::Closed(cms)=>{
            closed_model_scenario_scheduler(cms,journeys,scrapper).await
        },
        Scenario::Open(oms)=>{
            open_model_scenario_scheduler(oms,journeys,scrapper).await
        }
    }
}
async fn open_model_scenario_scheduler(scenario:ModelScenario,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>) {
    let stages= scenario.stages.clone();
    let mut threads = vec![];
    let mut vu =0;
    for stage in stages{
        let mut delta = stage.target;
        println!("Ramping up {} Iterations in {} seconds for test {}",delta,stage.duration,scenario.journey.clone());
        if delta!=0{
            let delay = stage.duration * 1000000 / (delta  as u64);
            for i in 0..delta{
                let th=start_iteration(scenario.journey.clone(),journeys.clone(),scrapper.clone()).await;
                threads.push(th);
                sleep(Duration::from_micros(delay)).await;
                vu = vu + 1;
            }
        }
        else {
            sleep(Duration::from_secs(stage.duration)).await;
        }
    }
    if let Some(ft) = &scenario.forceStop {
        tokio::select! {
            _=sleep(Duration::from_secs(ft.clone()))=>{println!("Forcefully stopped {}",scenario.journey)},
            _=futures::future::join_all(threads)=>{println!("Normally stopped {}",scenario.journey)}
        }
    } else {
        futures::future::join_all(threads).await;
        println!("Normally stopped {}",scenario.journey)
    }

}
async fn closed_model_scenario_scheduler(scenario:ModelScenario,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>) {
    let stages= scenario.stages.clone();
    let mut vus = vec![];
    let mut threads = vec![];
    let mut prev_num:i64 = 0;
    let mut vu =0;
    for stage in stages{
        let mut delta = (stage.target as i64) - prev_num;
        if delta >= 0 {
            println!("Ramping up {} VUs in {} seconds for test {}",delta,stage.duration,scenario.journey.clone());
            if delta!=0{
                let delay = stage.duration * 1000 / (delta  as u64);
                for i in 0..delta{
                    let (vuh,th)=start_vu(vu,scenario.journey.clone(),journeys.clone(),scrapper.clone()).await;
                    vus.push(vuh);
                    threads.push(th);
                    sleep(Duration::from_millis(delay)).await;
                    vu = vu + 1;
                    scrapper.ingest("vus",vu.clone() as f64,vec![("journey".to_string(),scenario.journey.clone())]).await;
                    // let result = client.insert_points(&points, TimestampOptions::None).await;
                }
            }
            else {
                let mut st=0;
                while st<stage.duration {
                    scrapper.ingest("vus",vu.clone() as f64,vec![("journey".to_string(),scenario.journey.clone())]).await;
                    sleep(Duration::from_secs(1)).await;
                    st =st +1;
                }
            }
        } else {
            println!("Ramping down {} VUs in {} seconds for test {}",delta*-1,stage.duration,scenario.journey.clone());
            let delay = stage.duration * 1000 / ((delta * -1) as u64);
            for i in 0..(delta*-1){
                if let Some(mut vu) = vus.pop(){
                    vu.send(1);
                }
                sleep(Duration::from_millis(delay)).await;
                vu = vu - 1;
                // let start = SystemTime::now();
                // let since_the_epoch = start
                //     .duration_since(UNIX_EPOCH)
                //     .expect("Time went backwards");
                // let point = VUS{
                //     name:scenario.journey.clone(),
                //     value: vu.clone(),
                //     timestamp:Timestamp::from(since_the_epoch.as_millis() as i64)
                // };
                // let points = vec![point];
                // let result = client.insert_points(&points, TimestampOptions::None).await;
                scrapper.ingest("vus",vu.clone() as f64,vec![("journey".to_string(),scenario.journey.clone())]).await;
            }

        }
        prev_num = stage.target as i64;
    }
    //Run Test For 25 sec
    for i  in 0..vus.len(){
        if let Some(mut vu) = vus.pop(){
            vu.send(1);
        }
    }
    if let Some(ft) = &scenario.forceStop {
        tokio::select! {
            _=sleep(Duration::from_secs(ft.clone()))=>{println!("Forcefully stopped {}",scenario.journey)},
            _=futures::future::join_all(threads)=>{println!("Normally stopped {}",scenario.journey)}
        }
    } else {
        futures::future::join_all(threads).await;
        println!("Normally stopped {}",scenario.journey)
    }
}
async fn start_vu(number:u64,name:String,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>)->(tokio::sync::mpsc::UnboundedSender<u64>,JoinHandle<()>){
    let (tx,mut rx) = tokio::sync::mpsc::unbounded_channel();
    let flag = Arc::new(RwLock::new(true));
    let name_clone=name.clone();
    let setter = async move |checker:Arc<RwLock<bool>>|{
        if let Some(val)=rx.recv().await{
            println!("Got Signal to Stop VU {} for test {}",number, name_clone);
            let mut flg = checker.write().await;
            *flg = false;
        }
    };
    let vu_loop = async move |checker:Arc<RwLock<bool>>|{
        let mut iteration = 0;
        loop {
            let tn = format!("{} - VU - {}, Iteration - {}",name.clone(),number,iteration);
            let flg = checker.read().await;
            if *flg {
                test(name.clone(),journeys.clone(),scrapper.clone()).await;
                iteration = iteration+1;
            } else {
                println!("Stopping VU {} for test {}",number, name.clone());
                break;
            }
        }
    };

    let h=tokio::spawn(async move {
        tokio::join!(setter(flag.clone()),vu_loop(flag.clone()));
    });
    (tx,h)
}
async fn start_iteration(name:String,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>)->JoinHandle<()>{
    tokio::spawn(test(name,journeys,scrapper))
}
async fn test(name:String,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>){
    let context = CorrContext::new(Arc::new(Mutex::new(StandAloneInterface{})),journeys.clone(),scrapper.clone());
    let now = Instant::now();
    let mut jn = Option::None;
    for jrn in &journeys {
        if jrn.name.eq(&name) {
            jn = Option::Some(jrn.clone());
            break;
        }
    }
    client::start(jn.unwrap(), Context::from_without_fallback(&context).await).await;
    scrapper.ingest("iterations",now.elapsed().as_millis() as f64,vec![("journey".to_string(),name)]).await;
}