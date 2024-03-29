use anyhow::bail;
use anyhow::Result;

use corr_lib::core::proto::{Input, Output};
use corr_lib::core::runtime::{Client, RuntimeError};
use async_trait::async_trait;
pub struct StandAloneInterface;
#[async_trait]
impl Client for StandAloneInterface {
    async fn send(&self, output: Output) ->Result<()>{
        match output {
            Output::KnowThat(kto)=>{
                println!("{}",kto.message);
            },
            Output::TellMe(tmo)=>{
                bail!(RuntimeError::new(format!("{} variable of type {:?} not defined",tmo.name,tmo.data_type).as_str()));
            }
            _=>{},
        };
        Ok(())
    }
    async fn get_message(&mut self) -> Result<Input> {
        bail!(RuntimeError::new("Some Variables are not defined!"));
    }
}