use corr_core::runtime::VariableDesciption;
use corr_core::runtime::RawVariableValue;
use serde::{Serialize, Deserialize};

#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub enum DesiredAction{
    Tell(VariableDesciption),
    Listen(String),
    Quit
}
#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub enum Action{
    Told(RawVariableValue),
    Quit,
    Ping,
    Pong,
    Ignorable
}


