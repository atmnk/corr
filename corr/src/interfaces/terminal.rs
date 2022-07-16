use tokio::sync::mpsc::{Receiver, Sender};
use tokio::io::{AsyncBufReadExt, BufReader, Lines, Stdin};
use async_trait::async_trait;
use corr_lib::core::proto::{Input, Output};
use corr_lib::core::runtime::{Client, RuntimeError};
use tokio::sync::mpsc;
use crate::client::Message;
use anyhow::{bail, Result};
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
    async fn send(&self,output:Output)->Result<()>{
         &self.tx.send(Message::Output(output)).await?;
        Ok(())
    }
    async fn get_message(&mut self)->Result<Input>{
        loop {
            if let Some(Message::Input(ip)) = self.rx.recv().await {
                return Ok(ip);
            } else {
                bail!(RuntimeError{
                    message:format!("Unable to read input")
                })
            }
        }
    }
}
