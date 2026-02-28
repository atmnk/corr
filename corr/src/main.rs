use std::str::FromStr;
use crate::launcher::{build, run};
use clap::{Parser, Subcommand};
use simple_error::SimpleError;

pub mod client;
pub mod launcher;
pub mod interfaces;
pub mod runners;

#[tokio::main]
async fn main() {
    let opt: Opts = Opts::parse();
    println!("{:?}", opt);
    match opt.command {
        SubCommands::Build {
            target,
            item,
            workload
        } => {
            build(target.clone(), item.clone(), workload).await.unwrap();
        }
        SubCommands::Run {
            debug,
            package,
            out,
            target,
            item,
            workload,
        } => {
            if package {
                run(target.clone(), item.clone(), !workload, out.clone(), debug).await
            } else {
                let target = build(target.clone(), item.clone(), workload.clone()).await.unwrap();
                run(target, item.clone(), !workload, out.clone(), debug).await
            }
        }
    };
}

#[derive(Parser, Debug)]
#[command(version, author = "Atmaram Naik <atmnk@yahoo.com>",about,long_about=None)]
struct Opts {
    #[command(subcommand)]
    command: SubCommands,
}

#[derive(Subcommand,Debug)]
enum SubCommands {
    #[clap(alias = "run")]
    Run {
        #[arg(short, long)]
        package: bool,

        #[arg(long, short, default_value = "console")]
        out: Out,

        #[arg(long, short, default_value = ".")]
        target: String,

        #[arg(short, long)]
        workload: bool,

        #[arg(short, long)]
        debug: bool,

        #[arg(default_value = "<default>")]
        item: String,

    },
    #[clap(alias = "build")]
    Build {
        #[arg(long, short, default_value = ".")]
        target: String,

        #[arg(short, long)]
        workload: bool,

        #[arg(default_value = "<default>")]
        item: String,
    },
}

#[derive(Debug, Clone)]
pub enum Out {
    InfluxDB2,
    Console,
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
// #[async_trait]
// pub trait Executable{
//     async fn execute(&self);
// }
// #[async_trait]
// impl Executable for BuildCommand{
//     async fn execute(&self) {
//         build(self.target.clone(),self.item.clone(),self.workload).await.unwrap();
//     }
// }
// #[async_trait]
// impl Executable for RunCommand{
//     async fn execute(&self) {
//         if self.package {
//             run(self.target.clone(), self.item.clone(), !self.workload, self.out.clone(),self.debug).await
//         } else {
//             let target = build(self.target.clone(),self.item.clone(),self.workload.clone()).await.unwrap();
//             run(target, self.item.clone(), !self.workload, self.out.clone(),self.debug).await
//         }
//     }
// }
