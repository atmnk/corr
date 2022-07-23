#![feature(generators, generator_trait)]
#![feature(async_closure)]
#![feature(backtrace)]
use std::str::FromStr;
use clap::Clap;
use clap::AppSettings;
use crate::launcher::{build, run};
use clap::{crate_version};
use async_trait::async_trait;
use simple_error::SimpleError;

pub mod client;
pub mod launcher;
pub mod interfaces;
pub mod runners;
#[tokio::main]
async fn main(){
    let opt: Opts = Opts::parse();
    println!("{:?}",opt);
    match opt {
        Opts::Build(bc)=>{
            bc.execute().await
        },
        Opts::Run(rc)=>{
            rc.execute().await
        }
    };
}
#[derive(Clap,Debug)]
#[clap(version = crate_version!(), author = "Atmaram Naik <atmnk@yahoo.com>",setting = AppSettings::InferSubcommands)]
enum Opts {
    #[clap(alias = "run")]
    Run(RunCommand),
    #[clap(alias = "build")]
    Build(BuildCommand),
}
#[derive(Clap,Debug)]
#[clap(version = crate_version!(), author = "Atmaram Naik <atmnk@yahoo.com>")]
pub struct RunCommand{
    #[clap(short, long)]
    package:bool,

    #[clap(long,short, default_value = "console")]
    out:Out,

    #[clap(long,short, default_value = ".")]
    target:String,

    #[clap(short, long)]
    workload:bool,

    #[clap(short, long)]
    debug:bool,

    #[clap(default_value = "<default>")]
    item:String,

}
#[derive(Debug,Clone)]
pub enum Out{
    InfluxDB2,
    Console
}
impl FromStr for Out {
    type Err = SimpleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "influxdb2" => Ok(Out::InfluxDB2),
            "console" => Ok(Out::Console),
            _ => Err(SimpleError::new("Invalid argument")),
        }
    }
}
#[derive(Clap,Debug)]
#[clap(version = crate_version!(), author = "Atmaram Naik <atmnk@yahoo.com>")]
pub struct BuildCommand{
    #[clap(long,short, default_value = ".")]
    target:String,

    #[clap(short, long)]
    workload:bool,

    #[clap(default_value = "<default>")]
    item:String,
}
#[async_trait]
pub trait Executable{
    async fn execute(&self);
}
#[async_trait]
impl Executable for BuildCommand{
    async fn execute(&self) {
        build(self.target.clone(),self.item.clone(),self.workload).await.unwrap();
    }
}
#[async_trait]
impl Executable for RunCommand{
    async fn execute(&self) {
        if self.package {
            run(self.target.clone(), self.item.clone(), !self.workload, self.out.clone(),self.debug).await
        } else {
            let target = build(self.target.clone(),self.item.clone(),self.workload.clone()).await.unwrap();
            run(target, self.item.clone(), !self.workload, self.out.clone(),self.debug).await
        }
    }
}
