use corr_lib::core::runtime::{Client, IO};
use corr_lib::core::runtime::Context as CorrContext;

use corr_lib::core::proto::{Input, Output};
use async_trait::async_trait;
pub struct CliDriver;
use flate2::read::GzDecoder;
use std::fs::{File, create_dir_all, remove_dir_all};
use tar::Archive;
use std::path::{Path, PathBuf};
use corr_lib::journey::{Journey, Executable};
use std::sync::Arc;
use futures::lock::Mutex;
use async_recursion::async_recursion;
use nom::error::convert_error;

use tokio::sync::{mpsc};
use tokio::sync::mpsc::{Receiver, Sender};



use corr_lib::parser::Parsable;

use tokio::io::{Stdin, BufReader, AsyncBufReadExt, Lines};



impl CliDriver{
    pub async fn run(target:String,journey:String){
        let jp=unpack(target).unwrap();
        Self::run_journey_in(jp,journey).await;
    }
    pub async fn run_journey_in(jp:String,journey:String){
        let jrns = get_journeis_in (format!("{}/src",jp)).await.unwrap();
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
                start(jn,context).await;
            });
            terminal.start().await;
        }
        // let (_,jrn) = Journey::parser(j.as_str()).unwrap();//Self::get_journey(jp,journey);


    }
}
pub async fn start(journey:Journey,context:CorrContext) {
    for param in journey.params.clone(){
        context.read(param).await;
    }
    let handles = journey.execute(&context).await;
    futures::future::join_all(handles).await;
    context.user.lock().await.send(Output::new_done("Done Executing Journey".to_string())).await;
}
fn unpack(target:String) -> Result<String, std::io::Error> {
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
pub struct Terminal{
    tx:Sender<Message>,
    rx:Receiver<Message>,
    reader:Lines<BufReader<Stdin>>
}
impl Terminal{
    pub fn new()->Self{
        let stdin = tokio::io::stdin();
        let reader = BufReader::new(stdin);
        let (tx,rx) = mpsc::channel(100);
        return Self {
            tx,
            rx,
            reader:reader.lines(),
        }
    }
    pub async fn start(&mut self){
        loop {
            if let Some(message) = self.rx.recv().await {
                if self.on_message(message).await {
                    return
                }
            }
        }
    }
    pub async fn on_message(&mut self,message:Message)->bool{
        match message {
            Message::Output(Output::KnowThat(kt))=>{
                println!("{}",kt.message);
                false
            },
            Message::Output(Output::TellMe(tm))=>{
                println!("Please Enter value for {} of type {:?}",tm.name,tm.data_type);
                let nl=self.reader.next_line().await.unwrap().unwrap();
                match self.tx.send(Message::Input(Input::new_continue(tm.name.clone(),nl,tm.data_type.clone()))).await
                {
                    Ok(_)=>{
                        false
                    },
                    Err(_e)=> {
                        true
                    }
                }
            },
            Message::Output(Output::Done(dom))=>{
                println!("{}",dom.message);
                true
            },
            _=>{unimplemented!()}
        }
    }
    pub fn get_if(&mut self)->CliInterface{
        let (tx,rx) = mpsc::channel(100);
        let (tx_s,rx_s) = mpsc::channel(100);
        self.rx = rx;
        self.tx = tx_s;
        return CliInterface{
            tx,
            rx:rx_s
        }
    }
}
pub struct CliInterface{
    tx:Sender<Message>,
    rx:Receiver<Message>
}
#[async_trait]
impl Client for CliInterface {
    async fn send(&self,output:Output){
        if let Err(_err) = &self.tx.send(Message::Output(output)).await {
            println!("Some Error")
        }
    }
    async fn get_message(&mut self)->Input{
        loop {
            if let Some(Message::Input(ip)) = self.rx.recv().await {
                return ip;
            }
        }
    }
}
#[async_recursion]
async fn get_journeis_in(path: impl AsRef<Path> + std::marker::Send + 'static)->tokio::io::Result<Vec<Journey>>{
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


