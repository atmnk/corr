use crate::template::VariableReferenceName;
use nom::combinator::{opt, not};
use nom::bytes::complete::tag;
use crate::core::runtime::Context;
use crate::core::{Variable, convert, DataType};
use nom::sequence::tuple;
use crate::template::text::extractable::parser::dynamic_tag;

pub mod parser;
#[derive(Clone,Debug,PartialEq)]
pub enum ExtractableText{
    Single(Variable),
    Multi(Option<String>,Vec<(Variable,String)>,Option<Variable>)
}

impl ExtractableText{
    pub async fn capture(&self,input:&str,context:&Context)->bool{
        match &self{
            Self::Single(v)=>{
                if let Some(val)=convert(v.name.clone(),input.to_string(),v.data_type.unwrap_or(DataType::String)){
                    context.define(val.name,val.value).await;
                    true
                } else {
                    false
                }
            },
            Self::Multi(first,vars,last)=>{
                let mut remaining = input.clone();
                let mut vars_to_define = vec![] ;
                if let Some(f) = first {
                    if let Some((i,t))=dynamic_tag (remaining,f.clone()).ok(){
                        remaining = i;
                    } else {
                        return false;
                    }
                }
                for (v,tag_s) in vars  {
                    let mut splitter = remaining.splitn(2,tag_s);
                    let first = splitter.next().unwrap();
                    if let Some(second) = splitter.next() {
                        if let Some(val) = convert(v.name.clone(),first.to_string(),v.data_type.unwrap_or(DataType::String)) {
                            vars_to_define.push(val);
                            remaining = second
                        } else {
                            return false;
                        }

                    }
                }
                if let Some(v) = last {
                    if let Some(val) = convert(v.name.clone(),remaining.to_string(),v.data_type.unwrap_or(DataType::String)) {
                        vars_to_define.push(val);
                    } else {
                        return false;
                    }
                };
                for val in vars_to_define{
                    context.define(val.name,val.value).await;
                }
                true
            }
        }

    }
}

