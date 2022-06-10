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
use corr_lib::core::scrapper::{Metrics, Scrapper};
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
}
pub async fn schedule_workload(workload:WorkLoad,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>){
    let joins:Vec<_> = workload.scenarios.iter().map(|sc|sc.clone()).map(|sc|schedule_scenario(sc,journeys.clone(),scrapper.clone())).collect();
    futures::future::join_all(joins).await;
}
async fn schedule_scenario(scenario:Scenario,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>){
    match scenario {
        Scenario::Closed(cms)=>{
            let metrics = Arc::new(Metrics::new());
            let jn = cms.journey.clone();
            tokio::select! {
                _=metrics_reporter(metrics.clone(),jn,scrapper.clone())=>{},
                _=closed_model_scenario_scheduler(cms,journeys,scrapper,metrics)=>{},
            }

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
async fn metrics_reporter(metrics:Arc<Metrics>,journey:String,scc:Arc<Box<dyn Scrapper>>){
    loop {
        scc.ingest_metric(metrics.clone(),("journey".to_string(),journey.clone())).await;
        sleep(Duration::from_millis(500)).await
    }
}
async fn closed_model_scenario_scheduler(scenario:ModelScenario,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>,metrics:Arc<Metrics>) {

    let stages= scenario.stages.clone();
    let mut vus = vec![];
    let mut threads = vec![];
    let mut prev_num:i64 = 0;
    let vu_count = Arc::new(RwLock::new(0 as f64));
    let mut vu =0;
    let vcc = vu_count.clone();
    let scc = scrapper.clone();
    let jnn = scenario.journey.clone();
    let jnnc = scenario.journey.clone();
    let metrics_reporter = async move ||{

    };
    for stage in stages{
        let mut delta = (stage.target as i64) - prev_num;
        if delta >= 0 {
            println!("Ramping up {} VUs in {} seconds for test {}",delta,stage.duration,scenario.journey.clone());
            if delta!=0{
                let delay = stage.duration * 1000 / (delta  as u64);
                for i in 0..delta{
                    let (vuh,th)=start_vu(vu,scenario.journey.clone(),journeys.clone(),scrapper.clone(),vu_count.clone(),metrics.clone()).await;
                    vus.push(vuh);
                    threads.push(th);
                    sleep(Duration::from_millis(delay)).await;
                    vu = vu + 1;
                    let count = vu_count.read().await;
                    {
                        let mut vuc = metrics.vus.write().await;
                        *vuc = *count;
                    }
                }
            }
            else {
                let mut st=0;
                while st<stage.duration {
                    let count = vu_count.read().await;
                    {
                        let mut vuc = metrics.vus.write().await;
                        *vuc = *count;
                    }
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
                let count = vu_count.read().await;
                {
                    let mut vuc = metrics.vus.write().await;
                    *vuc = *count;
                }
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
            _=metrics_reporter()=>{println!("VU Reporting stopped")},
            _=sleep(Duration::from_secs(ft.clone()))=>{println!("Forcefully stopped {}",scenario.journey)},
            _=futures::future::join_all(threads)=>{println!("Normally stopped {}",scenario.journey)}
        }
    } else {
        futures::future::join_all(threads).await;
        println!("Normally stopped {}",scenario.journey)
    }
}
async fn start_vu(number:u64,name:String,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>,vu_count:Arc<RwLock<f64>>,metrics:Arc<Metrics>)->(tokio::sync::mpsc::UnboundedSender<u64>,JoinHandle<()>){
    let (tx,mut rx) = tokio::sync::mpsc::unbounded_channel();
    let flag = Arc::new(RwLock::new(true));
    let name_clone=name.clone();
    let name_clone1 = name.clone();
    let setter = async move |checker:Arc<RwLock<bool>>|{
        if let Some(val)=rx.recv().await{
            println!("Got Signal to Stop VU {} for test {}",number, name_clone);
            let mut flg = checker.write().await;
            *flg = false;
        }
    };
    let scrapper1 = scrapper.clone();
    let name1 = name.clone();
    // let ping_vu = async move || {
    //     while true {
    //         scrapper1.ingest("vus",1.0,vec![("journey".to_string(),name1.clone())]).await;
    //         sleep(Duration::from_millis(1000)).await;
    //     }
    // };
    let im = Arc::new(RwLock::new((0,0.0)));
    let imc = im.clone();
    let vu_loop = async move |checker:Arc<RwLock<bool>>|{
        let mut iteration = 0;
        let vu_count = vu_count.clone();
        {
            let mut vc = vu_count.write().await;
            *vc = *vc + 1.0;
        }
        loop {
            let flg = checker.read().await;
            if *flg {
                let resp = test(name.clone(),journeys.clone(),scrapper.clone()).await;
                // {
                //     let  mut ic = im.write().await;
                //     *(ic) = (ic.0+1,((ic.0 as f64 * ic.1) +(resp as f64))/ (ic.0 as f64 + 1.0)) ;
                // }
                iteration = iteration+1;
                {
                    let mut itm = metrics.iterations.write().await;
                    let mut itmd = metrics.iteration_duration.write().await;
                    *itm = *itm + 1.0;
                    *itmd = *itmd + resp as f64;
                }
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
async fn start_iteration(name:String,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>)->JoinHandle<()>{
    let cc = async move ||{
        test(name,journeys,scrapper).await;
    };
    tokio::spawn(cc())
}
async fn test(name:String,journeys:Vec<Journey>,scrapper:Arc<Box<dyn Scrapper>>)->u128{
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
    now.elapsed().as_millis()
}