use corr_lib::core::runtime::{Client, IO};
use corr_lib::core::runtime::Context as CorrContext;

use corr_lib::core::proto::{Input, Output};
use async_trait::async_trait;
use flate2::read::GzDecoder;
use std::fs::{create_dir_all, File, remove_dir_all};
use tar::Archive;
use std::path::{Path, PathBuf};
use corr_lib::journey::{Executable, Journey};
use std::sync::Arc;
use futures::lock::Mutex;
use async_recursion::async_recursion;
use nom::error::convert_error;

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};



use corr_lib::parser::Parsable;

use tokio::io::{AsyncBufReadExt, BufReader, Lines, Stdin};
use corr_lib::workload::WorkLoad;
pub async fn start_internal(journey:Journey,mut context:CorrContext) {
    for param in journey.params.clone(){
        context.read(param).await;
    }
    let handles = journey.execute(&context).await;
    futures::future::join_all(handles).await;
}
pub async fn start(journey:Journey,mut context:CorrContext) {
    let mut rx = context.exiter();
    let user = context.user.clone();
    tokio::select! {
        _ = rx.recv() => {},
        _ = start_internal(journey,context) => {},
    }
    user.lock().await.send(Output::new_done("Done Executing Journey".to_string())).await;
}
pub fn unpack(target:String) -> Result<String, std::io::Error> {
    let tc = target.clone();
    let path = Path::new(tc.as_str());
    let name = path.file_stem();
    let tar_gz = File::open(target)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    remove_dir_all("./target");
    create_dir_all("./target")?;
    let jp = format!("./target/{}",name.unwrap().to_str().unwrap());
    archive.unpack(jp.clone())?;
    Ok(jp)
}
#[derive(Debug, Clone)]
pub enum Message{
    Input(Input),
    Output(Output)
}
#[async_recursion]
pub async fn get_workloads_in(path: impl AsRef<Path> + std::marker::Send + 'static)->tokio::io::Result<Vec<WorkLoad>>{
    let mut js:Vec<WorkLoad> = vec![];
    let mut dir = tokio::fs::read_dir(path).await?;
    while let Some(child) = dir.next_entry().await? {
        if child.metadata().await?.is_dir() {
            println!("Directory:{:?}",child.path());
            let mut child_j = get_workloads_in(child.path()).await?;
            js.append(&mut child_j)
        } else {
            let path:PathBuf = child.path();
            if let Some(Some(ext)) = path.extension().map(|ext|ext.to_str()) {
                if ext.to_lowercase().eq("workload") {
                    let text = tokio::fs::read_to_string(child.path()).await.unwrap();
                    let result = WorkLoad::parser(text.as_str());
                    match result {
                        Err(nom::Err::Error(er)) | Err(nom::Err::Failure(er))=>{
                            eprintln!("Unable to parse following errors {}",convert_error(text.as_str(),er))
                        },
                        Ok((_i,jrn))=>{
                            js.push(jrn);
                        },
                        _=>{
                            eprintln!("Some Other Error")
                        }
                    }

                }
            }
        }
    }
    Ok(js)
}
#[async_recursion]
pub async fn get_journeis_in(path: impl AsRef<Path> + std::marker::Send + 'static)->tokio::io::Result<Vec<Journey>>{
    let mut js:Vec<Journey> = vec![];
    let mut dir = tokio::fs::read_dir(path).await?;
    while let Some(child) = dir.next_entry().await? {
        if child.metadata().await?.is_dir() {
            println!("Directory:{:?}",child.path());
            let mut child_j = get_journeis_in(child.path()).await?;
            js.append(&mut child_j)
        } else {
            let path:PathBuf = child.path();
            if let Some(Some(ext)) = path.extension().map(|ext|ext.to_str()) {
                if ext.to_lowercase().eq("journey") {
                    let text = tokio::fs::read_to_string(child.path()).await.unwrap();
                    let result = Journey::parser(text.as_str());
                    match result {
                        Err(nom::Err::Error(er)) | Err(nom::Err::Failure(er))=>{
                            eprintln!("Unable to parse following errors {}",convert_error(text.as_str(),er))
                        },
                        Ok((_i,jrn))=>{
                            js.push(jrn);
                        },
                        _=>{
                            eprintln!("Some Other Error")
                        }
                    }

                }
            }
        }
    }
    Ok(js)
}


