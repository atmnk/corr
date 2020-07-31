pub mod text;
use crate::core::{DataType, Context, Value};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Clone,Debug)]
pub enum Expression{
    Variable(String,Option<DataType>),
    Function(Arc<dyn Function>,Vec<Expression>),
}
pub trait Function:Debug+Sync+Send{
    fn evaluate(&self,args:Vec<Expression>,context:&Context)->Value;
}