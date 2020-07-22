use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures::lock::Mutex;
use std::collections::HashMap;
use futures::Future;
use async_trait::async_trait;
use crate::core::proto::{Output, Input};

pub mod proto;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum DataType {
    String,
    Double,
    Long,
    Boolean
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Value{
    String(String),
    Long(usize)
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableValue{
    pub name:String,
    pub value:Value
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variable{
    pub name:String,
    pub data_type:DataType
}
pub fn convert(name:String,value:String,data_type:DataType)->Option<VariableValue>{
    match data_type {
        DataType::String=>Option::Some(VariableValue{name,value:Value::String(value)}),
        DataType::Long=>{
            if let Ok(val) = value.parse::<usize>(){
                Option::Some(VariableValue{name,value:Value::Long(val)})
            } else {
                Option::None
            }
        },
        _=>Option::None
    }
}
pub enum HeapObject{
    Final(Value),
    List(Vec<Arc<Mutex<HeapObject>>>),
    Object(HashMap<String,Arc<Mutex<HeapObject>>>)
}
#[derive(Debug, Clone)]
pub struct ReferenceStore{
    references:Arc<Mutex<HashMap<String,Arc<Mutex<HeapObject>>>>>
}
impl ReferenceStore{
    pub fn new()->Self{
        ReferenceStore{
            references:Arc::new(Mutex::new(HashMap::new()))
        }
    }
    pub async fn from(rs:&ReferenceStore)->Self{
        return ReferenceStore{
            references:Arc::new(Mutex::new(rs.references.lock().await.clone()))
        }
    }
    pub async fn set(&self,path:String,value:Arc<Mutex<HeapObject>>){
        self.references.lock().await.insert(path,value);
    }

    pub async fn get(&self,path:String)->Option<Arc<Mutex<HeapObject>>>{
        if let Some(arc) = self.references.lock().await.get(&path){
            Some(arc.clone())
        } else {
            None
        }
    }
}

#[async_trait]
impl IO for Context {
    async fn write(&self, data:String){
        self.user.lock().await.send(Output::new_know_that(data));
    }

    async fn read(&self, variable: Variable)->VariableValue{
        self.user.lock().await.send(Output::new_tell_me(variable.name.clone(),variable.data_type.clone()));
        loop{
            let message=self.user.lock().await.get_message().await;
            if let Some(var) =match message {
                Input::Continue(continue_input)=>continue_input.convert(),
                _=>Option::None
            }{
                if var.name.eq(&variable.name){
                    return var;
                } else {
                    continue;
                }
            } else {
                self.user.lock().await.send(Output::new_know_that(format!("Invalid Value")));
                self.user.lock().await.send(Output::new_tell_me(variable.name.clone(),variable.data_type.clone()));
            }
        }
    }
}
pub struct Context{
    pub user:Arc<Mutex<dyn Client>>,
    pub store:ReferenceStore,
}
impl Context {
    pub async fn from(context:&Context)->Self{
        Context{
            user:context.user.clone(),
            store:ReferenceStore::from(&context.store).await
        }
    }
    pub async fn iterate<F, Fut>(&self,path:String,temp:String,iterateThis: F)
        where
            F: FnOnce(Context) -> Fut + Copy,
            Fut: Future<Output = ()>,
    {
        if let Some(arc) = self.store.get(path).await{
            if let HeapObject::List(lst) = &*arc.lock().await {
                for l in lst {
                    let new_ct = Context::from(self).await;
                    new_ct.store.set(temp.clone(),l.clone()).await;
                    iterateThis(new_ct).await;
                }
            }
        }
    }
}
#[async_trait]
pub trait Client:Send{
    fn send(&self,output:Output);
    async fn get_message(&mut self)->Input;
}
#[async_trait]
pub trait IO {
    async fn write(&self,data:String);
    async fn read(&self,variable:Variable)->VariableValue;
}
#[cfg(test)]
mod tests{
    use crate::core::{ReferenceStore, HeapObject, Value};
    use std::sync::Arc;
    use futures::lock::Mutex;


    // #[tokio::test]
    // async fn should_iterate(){
    //     let rs=;
    //     rs.set(format!("test"),Arc::new(Mutex::new(HeapObject::List(vec![Arc::new(Mutex::new(HeapObject::Final(Value::Long(12)))),Arc::new(Mutex::new(HeapObject::Final(Value::Long(12))))])))).await;
    //     rs.iterate(format!("test"),format!("obj"),async move |rs1|{
    //         rs1.set(format!("pq"),Arc::new(Mutex::new(HeapObject::Final(Value::Long(10))))).await
    //     }).await;
    // }
}
