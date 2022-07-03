use std::collections::HashMap;
use std::fs::{File, read_to_string, create_dir_all, remove_dir_all};
use flate2::Compression;
use flate2::write::GzEncoder;
use serde::{Deserialize};
use crate::runners::journey::JourneyRunner;
use std::path::Path;
use std::sync::Arc;
use corr_lib::journey::{Executable, Journey};
use crate::client::{get_journeis_in, get_workloads_in};
use crate::Out;
use crate::runners::workload::WorkLoadRunner;
use async_recursion::async_recursion;
use corr_lib::workload::Scenario;

pub async fn build(target:String,root:String,is_workload:bool)-> Result<String, std::io::Error>{
    pack(target,root,is_workload).await
}
#[derive(Deserialize)]
pub struct Config {
    package:Package
}
#[derive(Deserialize)]
struct Package {
    name: String,
}
async fn pack(target:String,root:String,is_workload:bool) -> Result<String, std::io::Error> {
    let toml = format!("{}/jpack.toml",target);
    let mut config:Config = Config {
        package:Package{
            name:"temp".to_string()
        }
    };
    if Path::new(toml.as_str()).exists() {
        config = toml::from_str(read_to_string(toml).unwrap().as_str()).unwrap();
    }
    let _ = remove_dir_all(format!("{}/build/src",target));
    create_dir_all(format!("{}/build/src",target))?;
    let result = format!("{}/build/{}.jpack",target,config.package.name.clone());
    let tar_gz = File::create(result.clone())?;
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    copy_dependencies_in(format!("{}/build/src", target), format!("{}/src", target), root, is_workload).await;
    tar.append_dir_all("./src", format!("{}/build/src",target))?;
    Ok(result)
}
pub async fn run(target:String, item:String, is_journey:bool, out:Out,debug:bool){
    if is_journey
    {
        JourneyRunner::run(target, item,out,debug).await;
    } else {
        WorkLoadRunner::run(target,item,out,debug).await;
    }
}
pub async fn copy_dependencies_in(target_dir:String, source:String, item:String, is_workload:bool){
    let jrns = get_journeis_in(source.clone(),"".to_string()).await.unwrap();
    let jrns_arc = Arc::new(jrns);
    if is_workload {
        settle_workload(target_dir, source, item, jrns_arc).await;
    } else {
        settle_journey(target_dir, source, item, jrns_arc).await;
    }
}
async fn settle_workload(target_dir: String, source: String, item: String, jrns_arc: Arc<HashMap<String, Arc<Journey>>>) {
    let mut p: Vec<String> = item.split(".").map(|s| s.to_string()).collect();
    let name = p.pop().unwrap();
    let path = p.join("/");
    let wklds = get_workloads_in(source.clone()).await.unwrap();
    let wlo = wklds.iter().find(|w|w.name.eq(&item));
    if let Some(wl) = wlo {
        create_dir_all(format!("{}/{}", target_dir, path)).unwrap();
        tokio::fs::copy(format!("{}/{}/{}.workload", source, path, name), format!("{}/{}/{}.workload", target_dir, path, name)).await.unwrap();
        if let Some(s) = &wl.setup {
            settle_journey(target_dir.clone(), source.clone(), s.clone(), jrns_arc.clone()).await;
        }
        for sc in &wl.scenarios {
            match sc {
                Scenario::Open(ms)=>{
                    settle_journey(target_dir.clone(), source.clone(), ms.journey.clone(), jrns_arc.clone()).await;
                },
                Scenario::Closed(ms)=>{
                    settle_journey(target_dir.clone(), source.clone(), ms.journey.clone(), jrns_arc.clone()).await;
                }
            }
        }

    } else {
        eprintln!("Workload {} not found",item)
    }
}
async fn settle_journey(target_dir: String, source: String, item: String, jrns_arc: Arc<HashMap<String, Arc<Journey>>>) {
    let mut p: Vec<String> = item.split(".").map(|s| s.to_string()).collect();
    let name = p.pop().unwrap();
    let path = p.join("/");
    create_dir_all(format!("{}/{}", target_dir, path)).unwrap();
    tokio::fs::copy(format!("{}/{}/{}.journey", source, path, name), format!("{}/{}/{}.journey", target_dir, path, name)).await.unwrap();
    let deps = get_total_dependencies(vec![], item, jrns_arc).await;
    for dep in deps {
        let mut p: Vec<String> = dep.clone().split(".").map(|s| s.to_string()).collect();
        let name = p.pop().unwrap();
        let path = p.join("/");
        create_dir_all(format!("{}/{}", target_dir, path)).unwrap();
        tokio::fs::copy(format!("{}/{}/{}.journey", source, path, name), format!("{}/{}/{}.journey", target_dir, path, name)).await.unwrap();
    }
}

#[async_recursion]
pub async fn get_total_dependencies(path:Vec<String>, journey:String, journeys:Arc<HashMap<String,Arc<Journey>>>) ->Vec<String>{
    let mut ads = vec![];
    let jrn = journeys.get(&journey).map(|j|j.clone());
    if let Some(j) = jrn {
        let deps = j.get_deps();
        for dep in &deps {
            if path.contains(dep){
                panic!("Circular Dependency {}->{}->{}",path.join("->"),journey,dep)
            } else {
                ads.push(dep.clone());
                let mut new_path = path.clone();
                new_path.push(journey.clone());
                let mut ad = get_total_dependencies(new_path, dep.clone(), journeys.clone()).await;
                ads.append(&mut ad)
            }
        }

    }
    ads
}