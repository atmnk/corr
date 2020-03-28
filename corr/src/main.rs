extern crate clap;
extern crate corr_client_lib;
extern crate app_dirs2;
use app_dirs2::*;
use clap::{Arg, App};
use std::fs::File;
use std::io::prelude::*;
use corr_client_lib::*;
const APP_INFO: AppInfo = AppInfo{name: "corr", author: "Atmaram Naik"};
fn main()->std::io::Result<()>{
    let matches = App::new("Correlate Api's")
        .version("0.1.1")
        .author("Atmaram R. Naik <naik.atmaram@gmail.com>")
        .about("Command line tool for api journey processing")
        .subcommand(
            App::new("connect")
                .about("Connect app with server")
                .version("0.1.0")
                .author("Atmaram R. Naik <naik.atmaram@gmail.com>")
                .arg(
                    Arg::with_name("server")
                        .index(1)
                        .required(true)
                )
        )
        .subcommand(
            App::new("run")
                .about("Run Journey")
                .version("0.1.0")
                .author("Atmaram R. Naik <naik.atmaram@gmail.com>")
                .arg(Arg::with_name("filter").help("sets env for journey")
                    .takes_value(true)
                    .short("f")
                    .long("filter")
                )
        )
        .get_matches();


    // You can get the independent subcommand matches (which function exactly like App matches)
    if let Some(ref matches) = matches.subcommand_matches("connect") {
        let mut path=app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
        path.set_file_name("corr/server.txt");
        let mut file = File::create(path)?;
        file.write_all(matches.value_of("server").unwrap().as_bytes())?;
        return Ok(());
    }

    if let Some(ref matches) = matches.subcommand_matches("run") {
        let mut path=app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
        path.set_file_name("corr/server.txt");
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let mut client = CliClient::new(contents.clone());
        let filter = Filter{
            value:matches.value_of("filter").unwrap().to_string()
        };
        match client.run(filter) {
            Ok(_)=>{
                println!("You are done here")
            },
            Err(err)=>{
                println!("error{:?}",err)
            }
        };
        return Ok(());
    }
    return Ok(());
}