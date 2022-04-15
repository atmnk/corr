#![feature(generators, generator_trait)]
use clap::Clap;
use clap::AppSettings;
use crate::launcher::{build, run};
use clap::{crate_version};
use async_trait::async_trait;
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

    #[clap(long,short, default_value = ".")]
    target:String,

    #[clap(short, long)]
    isWorkload:bool,

    #[clap(default_value = "<default>")]
    item:String,

}

#[derive(Clap,Debug)]
#[clap(version = crate_version!(), author = "Atmaram Naik <atmnk@yahoo.com>")]
pub struct BuildCommand{
    #[clap(default_value = ".")]
    target:String,
}
#[async_trait]
pub trait Executable{
    async fn execute(&self);
}
#[async_trait]
impl Executable for BuildCommand{
    async fn execute(&self) {
        build((&self.target).clone()).unwrap();
    }
}
#[async_trait]
impl Executable for RunCommand{
    async fn execute(&self) {
        if self.package {
            run(self.target.clone(), self.item.clone(), !self.isWorkload).await
        } else {
            let target = build((&self.target).clone()).unwrap();
            run(target, self.item.clone(), !self.isWorkload).await
        }
    }
}
