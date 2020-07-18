use serde::{Deserialize, Serialize};
use std::result;
use warp::Error;
use crate::core::{DataType, convert, VariableValue};

pub type Result<T> = result::Result<T, Error>;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Input {
    #[serde(rename = "start")]
    Start(StartInput),
    #[serde(rename = "continue")]
    Continue(ContinueInput),
}

impl Input{
    pub fn new_start(filter:String)->Self{
        Input::Start(StartInput{filter})
    }
    pub fn new_continue(name:String,value:String,data_type:DataType)->Self{
        Input::Continue(ContinueInput{name,value,data_type})
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum Output {
    #[serde(rename = "knowThat")]
    KnowThat(KnowThatOutput),
    #[serde(rename = "tellMe")]
    TellMe(TellMeOutput),
}
impl Output{
    pub fn new_know_that(message:String)->Self{
        Output::KnowThat(KnowThatOutput{message})
    }
    pub fn new_tell_me(name:String,data_type:DataType)->Self{
        Output::TellMe(TellMeOutput{name,data_type})
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartInput {
    pub filter: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContinueInput {
    pub name:String,
    pub value:String,
    pub data_type:DataType
}
impl ContinueInput {
    pub fn convert(&self)->Option<VariableValue>{
        convert(self.name.clone(),self.value.clone(),self.data_type.clone())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowThatOutput {
    pub message: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TellMeOutput {
    pub name:String,
    pub data_type:DataType
}
