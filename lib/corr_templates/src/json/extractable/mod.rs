use super::corr_core::runtime::Variable;
use crate::json::Json;
use std::collections::HashMap;
pub mod parser;
#[derive(Clone,PartialEq,Debug)]
pub struct CaptuarableArray{
    as_var:Variable,
    in_var:Variable,
    inner_json:Box<ExtractableJson>
}
#[derive(Clone,PartialEq,Debug)]
pub enum ExtractableJson {
    Variable(Variable),
    TemplatedStaticArray(Vec<ExtractableJson>),
    TemplatedDynamicArray(CaptuarableArray),
    Object(HashMap<String,ExtractableJson>),
}