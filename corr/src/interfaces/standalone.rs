use backtrace::Backtrace;
use corr_lib::core::proto::{Input, Output};
use corr_lib::core::runtime::Client;
use async_trait::async_trait;
pub struct StandAloneInterface;
#[async_trait]
impl Client for StandAloneInterface {
    async fn send(&self, output: Output) {
        match output {
            Output::KnowThat(kto)=>{
                println!("{}",kto.message);
            },
            _=>{},
        };
    }
    async fn get_message(&mut self) -> Input {
        panic!("Some Variables are not defined! {:?}",Backtrace::new());
    }
}