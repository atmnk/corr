use std::env;
use std::sync::Arc;
use std::time::{Duration};
use futures::lock::Mutex;
use core::option::Option;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep};
use crate::{client, Out};
use crate::interfaces::standalone::StandAloneInterface;
use corr_lib::core::runtime::{Context as CorrContext, Context};
use corr_lib::core::scrapper::influxdb2::InfluxDB2Scrapper;
use corr_lib::core::scrapper::none::NoneScraper;
use corr_lib::core::scrapper::{Scrapper};

use corr_lib::journey::{Journey};
use corr_lib::workload::{ModelScenario, Scenario, WorkLoad};
pub struct WorkLoadRunner;
impl WorkLoadRunner{
    pub async fn run(target:String,workload:String,out:Out,debug:bool){
        let jp= client::unpack(target).unwrap();
        Self::run_workload_in(jp, workload,out,debug).await;
    }
    pub async fn run_workload_in(jp:String, workload:String,out:Out,debug:bool){
        let wrklds = client::get_workloads_in(format!("{}/src", jp),"".to_string()).await.unwrap();
        let jrns = client::get_journeis_in(format!("{}/src", jp),"".to_string()).await.unwrap();
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
            schedule_workload(wl,jrns,Arc::new(scrp),debug).await
        }
    }
}
pub async fn schedule_workload(workload:WorkLoad, journeys:HashMap<String,Arc<Journey>>, scrapper:Arc<Box<dyn Scrapper>>, debug:bool){
    let context = CorrContext::new(Arc::new(Mutex::new(StandAloneInterface{})),journeys.clone(),scrapper.clone(),debug);
    let mut cont = true;
    if let Some(setup) = &workload.setup{
        if let Some(jn) = journeys.get(setup) {
            client::start(jn.clone(), context.clone()).await;
        } else {
            cont = false;
            eprintln!("Runtime Error: Setup Journey {} not found exiting execution",setup);
        }

    }
    if cont {
        let joins:Vec<_> = workload.scenarios.iter().map(|sc|sc.clone()).map(|sc|schedule_scenario(sc, journeys.clone(), scrapper.clone(), context.clone(), debug)).collect();
        tokio::select! {
            _= scrapper.start_metrics_loop()=>{},
            _= futures::future::join_all(joins)=>{}
        }
    }
}
async fn schedule_scenario(scenario:Scenario, journeys:HashMap<String,Arc<Journey>>, scrapper:Arc<Box<dyn Scrapper>>, context:CorrContext, debug:bool){
    let count = Arc::new(RwLock::new(0.0));
    let cc = count.clone();
    let scpr = scrapper.clone();
    let iteration_scrapper = async move |jn:String|{
        loop {
            let mut ct = count.write().await;
            scpr.ingest("iteration_count",*ct,vec![("journey".to_string(),jn.clone())]).await;
            *ct = 0.0;
            sleep(Duration::from_millis(50)).await
        }
    };
    match scenario {
        Scenario::Closed(cms)=>{
            let jn = cms.journey.clone();
            tokio::select! {
                _=closed_model_scenario_scheduler(cms,journeys,scrapper,cc,context.clone(),debug)=>{},
                _=iteration_scrapper(jn)=>{},
            };
        },
        Scenario::Open(oms)=>{
            let jn = oms.journey.clone();
            tokio::select! {
                _=open_model_scenario_scheduler(oms,journeys,scrapper,cc,context.clone(),debug)=>{},
                _=iteration_scrapper(jn)=>{},
            };
        }
    }
}
async fn open_model_scenario_scheduler(scenario:ModelScenario, journeys:HashMap<String,Arc<Journey>>, scrapper:Arc<Box<dyn Scrapper>>, ic:Arc<RwLock<f64>>, context:CorrContext, debug:bool) {
    if debug {
        start_iteration(scenario.journey.clone(),journeys.clone(),scrapper.clone(),ic.clone(),context.clone()).await.await.unwrap();
    } else {
        let stages= scenario.stages.clone();
        let mut threads = vec![];
        let _vu =0;
        let mut prev = 0;
        let mut last_stage = 0;
        for stage in stages{
            for j in 0..stage.duration {
                if stage.target > prev {
                    prev = last_stage + ((j+1) as f64*((stage.target - last_stage) as f64 / stage.duration as f64)) as u64;
                } else {
                    prev = last_stage - ((j+1) as f64 *(( last_stage - stage.target) as f64 / stage.duration as f64)) as u64;
                }
                if prev!=0{
                    let nowo = Instant::now();
                    for _i in 0..(prev){
                        let th=start_iteration(scenario.journey.clone(),journeys.clone(),scrapper.clone(),ic.clone(),context.clone()).await;
                        threads.push(th);
                    }
                    let elo = nowo.elapsed().as_micros() as u64;
                    if elo < 1000000 {
                        sleep(Duration::from_micros(1000000-elo)).await;
                    }
                } else {
                    sleep(Duration::from_millis(1000)).await;
                }

            }
            last_stage = stage.target;
        }
        if let Some(ft) = &scenario.force_stop {
            tokio::select! {
            _=sleep(Duration::from_secs(ft.clone()))=>{println!("Forcefully stopped {}",scenario.journey)},
            _=futures::future::join_all(threads)=>{println!("Normally stopped {}",scenario.journey)}
        }
        } else {
            futures::future::join_all(threads).await;
            println!("Normally stopped {}",scenario.journey)
        }
    }

}
async fn closed_model_scenario_scheduler(scenario:ModelScenario, journeys:HashMap<String,Arc<Journey>>, scrapper:Arc<Box<dyn Scrapper>>, ic:Arc<RwLock<f64>>, context:CorrContext, debug:bool) {

    let stages= scenario.stages.clone();
    let mut vus = vec![];
    let mut threads = vec![];
    let mut prev_num:i64 = 0;
    let vu_count = Arc::new(RwLock::new(0 as f64));
    let mut vu =0;
    let _vcc = vu_count.clone();
    let _scc = scrapper.clone();
    let jnn = scenario.journey.clone();
    let _jnnc = scenario.journey.clone();
    if debug {
        start_iteration(scenario.journey.clone(),journeys.clone(),scrapper.clone(),ic.clone(),context.clone()).await.await.unwrap();
    } else {
        for stage in stages{
            let delta = (stage.target as i64) - prev_num;
            if delta >= 0 {
                println!("Ramping up {} VUs in {} seconds for test {}",delta,stage.duration,scenario.journey.clone());
                if delta!=0{
                    let delay = stage.duration * 1000 / (delta  as u64);
                    for _i in 0..delta{
                        let (vuh,th)=start_vu(vu,scenario.journey.clone(),journeys.clone(),scrapper.clone(),vu_count.clone(),ic.clone(),context.clone()).await;
                        vus.push(vuh);
                        threads.push(th);
                        sleep(Duration::from_millis(delay)).await;
                        vu = vu + 1;
                        let count = vu_count.read().await;
                        scrapper.ingest("vus",*count,vec![("jounrey".to_string(),jnn.clone())]).await;
                    }
                }
                else {
                    let mut st=0;
                    while st<stage.duration {
                        let count = vu_count.read().await;
                        scrapper.ingest("vus",*count,vec![("jounrey".to_string(),jnn.clone())]).await;
                        sleep(Duration::from_secs(1)).await;
                        st =st +1;
                    }
                }
            } else {
                println!("Ramping down {} VUs in {} seconds for test {}",delta*-1,stage.duration,scenario.journey.clone());
                let delay = stage.duration * 1000 / ((delta * -1) as u64);
                for _i in 0..(delta*-1){
                    if let Some(vu) = vus.pop(){
                        let _ = vu.send(1);
                    }
                    sleep(Duration::from_millis(delay)).await;
                    let count = vu_count.read().await;
                    scrapper.ingest("vus",*count,vec![("jounrey".to_string(),jnn.clone())]).await;
                }

            }
            prev_num = stage.target as i64;
        }
        for _i  in 0..vus.len(){
            if let Some(vu) = vus.pop(){
                let _ = vu.send(1);
            }
        }
        if let Some(ft) = &scenario.force_stop {
            tokio::select! {
            _=sleep(Duration::from_secs(ft.clone()))=>{println!("Forcefully stopped {}",scenario.journey)},
            _=futures::future::join_all(threads)=>{println!("Normally stopped {}",scenario.journey)}
        }
        } else {
            futures::future::join_all(threads).await;
            println!("Normally stopped {}",scenario.journey)
        }
    }

}
async fn start_vu(number:u64,name:String,journeys:HashMap<String,Arc<Journey>>,scrapper:Arc<Box<dyn Scrapper>>,vu_count:Arc<RwLock<f64>>,ic:Arc<RwLock<f64>>,context:CorrContext)->(tokio::sync::mpsc::UnboundedSender<u64>,JoinHandle<()>){
    let (tx,mut rx) = tokio::sync::mpsc::unbounded_channel();
    let flag = Arc::new(RwLock::new(true));
    let name_clone=name.clone();
    let setter = async move |checker:Arc<RwLock<bool>>|{
        if let Some(_val)=rx.recv().await{
            println!("Got Signal to Stop VU {} for test {}",number, name_clone);
            let mut flg = checker.write().await;
            *flg = false;
        }
    };
    let _im = Arc::new(RwLock::new((0,0.0)));
    let vu_loop = async move |checker:Arc<RwLock<bool>>|{
        let mut iteration = 0;
        let vu_count = vu_count.clone();
        {
            let mut vc = vu_count.write().await;
            *vc = *vc + 1.0;
        }
        let mut total_resp = 0;
        let mut intc:f64 = 0.0;
        loop {
            let flg = checker.read().await;
            if *flg {
                let resp = test(name.clone(),journeys.clone(),scrapper.clone(),context.clone()).await;
                total_resp = total_resp + resp;
                intc = intc+1.0;
                scrapper.ingest("iteration_duration",resp as f64,vec![("journey".to_string(),name.clone())]).await;
                if total_resp >= 500 {
                    let mut ic_ref = ic.write().await;
                    *ic_ref = *ic_ref + intc;
                    intc = 0.0;
                    total_resp = 0;
                }
                iteration = iteration+1;
            } else {
                println!("Stopping VU {} for test {}",number, name.clone());
                break;
            }
        }
        {
            let mut vc = vu_count.write().await;
            *vc = *vc - 1.0;
        }
    };
    let flag1 = flag.clone();
    let h=tokio::spawn(async move {
        tokio::join!(setter(flag.clone()),vu_loop(flag1));
    });
    (tx,h)
}
async fn start_iteration(name:String,journeys:HashMap<String,Arc<Journey>>,scrapper:Arc<Box<dyn Scrapper>>,ic:Arc<RwLock<f64>>,context:CorrContext)->JoinHandle<()>{
    let cc = async move ||{
        let resp = test(name.clone(),journeys,scrapper.clone(),context.clone()).await;
        scrapper.ingest("iteration_duration",resp as f64,vec![("journey".to_string(),name.clone())]).await;
        {
            let mut ic_ref = ic.write().await;
            *ic_ref = *ic_ref + 1.0;
        }
    };
    tokio::spawn(cc())
}
async fn test(name:String,journeys:HashMap<String,Arc<Journey>>,_scrapper:Arc<Box<dyn Scrapper>>,context:CorrContext)->u128{
    let context = CorrContext::copy_from(&context).await;//CorrContext::new(Arc::new(Mutex::new(StandAloneInterface{})),journeys.clone(),scrapper.clone());
    let now = Instant::now();
    client::start(journeys.get(&name).unwrap().clone(), context).await;
    now.elapsed().as_millis()
}