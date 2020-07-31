use serde::{Deserialize, Serialize};
use std::sync::Arc;
use futures::lock::Mutex;
use std::collections::HashMap;
use futures::Future;
use async_trait::async_trait;
use crate::core::proto::{Output, Input};
use async_recursion::async_recursion;
pub mod proto;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum DataType {
    String,
    Double,
    Long,
    Boolean,
    List,
    Object
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum Value{
    String(String),
    Long(usize),
    Boolean(bool),
    Double(f64),
    Null
}
impl Value {
    pub fn is_of_type(&self,data_type:DataType)->bool{
        match self {
            Value::Null=>true,
            _=>{
                match data_type {
                    DataType::Long=> if let Value::Long(_)=self{
                        true
                    } else {
                        false
                    },
                    DataType::String=> if let Value::String(_)=self{
                        true
                    } else {
                        false
                    },
                    DataType::Boolean=> if let Value::Boolean(_)=self{
                        true
                    } else {
                        false
                    },
                    DataType::Double=> if let Value::Double(_)=self{
                        true
                    } else {
                        false
                    },
                    _=> false
                }
            }
        }

    }
    pub fn to_string(&self)->String{
        match self {
            Value::String(str)=>str.clone(),
            Value::Null=>"null".to_string(),
            Value::Long(lng)=>format!("{}",lng),
            Value::Double(dbl)=>format!("{}",dbl),
            Value::Boolean(bln)=>format!("{}",bln)
        }
    }
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
    pub data_type:Option<DataType>
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
impl HeapObject{
    pub fn from(val:Value)->Self{
        HeapObject::Final(val)
    }
    pub fn is_of_type(&self,data_type:DataType)->bool{
        match self {
            HeapObject::Final(val)=>{
                val.is_of_type(data_type.clone())
            },
            HeapObject::List(_)=>{
                match &data_type {
                    DataType::List=>true,
                    _=>false,
                }
            },
            HeapObject::Object(_)=>{
                match &data_type {
                    DataType::List=>true,
                    _=>false,
                }
            }
        }
    }
    pub fn to_value(&self)->Option<Value>{
        match self {
            HeapObject::Final(val)=>{
                Option::Some(val.clone())
            },
            _=>{Option::None}
        }
    }
}
#[derive(Debug, Clone)]
pub struct ReferenceStore{
    parent:Option<Box<ReferenceStore>>,
    references:Arc<Mutex<HashMap<String,Arc<Mutex<HeapObject>>>>>
}
pub fn break_on(path:String,chr:char)->Option<(String,String)>{
    let spl:Vec<&str>=path.rsplitn(2,chr).collect();
    if spl.len() == 2{
        Option::Some((spl[1].to_string(),spl[0].to_string()))
    }
    else {
        Option::None
    }
}
#[async_recursion]
pub async fn get_value_from(path:String,heap_object_ref:Arc<Mutex<HeapObject>>)->Option<Arc<Mutex<HeapObject>>>{
    let ho = &*heap_object_ref.lock().await;
    match ho {
        HeapObject::Object(obj)=>{
            if let Some((left,right))=break_on(path.clone(),'.'){
                if let Some(key_value)=obj.get(&left){
                    get_value_from(right.clone(),key_value.clone()).await
                } else {
                    Option::None
                }

            } else {
                if let Some(key_vale)=obj.get(&path.clone()){
                    Option::Some(key_vale.clone())
                } else {
                    Option::None
                }
            }
        },
        _=> Option::None
    }
}
#[async_recursion]
pub async fn set_value_at(path:String,heap_object_ref:Arc<Mutex<HeapObject>>,value:Arc<Mutex<HeapObject>>)->Option<Arc<Mutex<HeapObject>>>{
    let ho = &mut *heap_object_ref.lock().await;
    match ho {
        HeapObject::Object(obj)=>{
            if let Some((left,right))=break_on(path.clone(),'.'){
                if let Some(key_value)=obj.get(&left){
                    set_value_at(right.clone(),key_value.clone(),value).await
                } else {
                    Option::None
                }
            } else {
                obj.insert(path.clone(),value.clone());
                Option::Some(value)
            }
        },
        _=> Option::None
    }
}
impl ReferenceStore{
    pub fn new()->Self{
        ReferenceStore{
            parent:Option::None,
            references:Arc::new(Mutex::new(HashMap::new()))
        }
    }
    pub async fn from(rs:&ReferenceStore)->Self{
        return ReferenceStore{
            parent:Option::Some(Box::new(rs.clone())),
            references:Arc::new(Mutex::new(rs.references.lock().await.clone()))
        }
    }
    #[async_recursion]
    pub async fn set(&self,path:String,value:Arc<Mutex<HeapObject>>){
        if let Some((left,right)) = break_on(path.clone(),'.'){
            if let Some(arc) = self.references.lock().await.get(&left){
                set_value_at(right.clone(),arc.clone(),value).await;
            } else {
                let obj = Arc::new(Mutex::new(HeapObject::Object(HashMap::new())));
                set_value_at(right.clone(),obj.clone(),value).await;
                self.references.lock().await.insert(left.clone(),obj);
            }
        } else {
            self.references.lock().await.insert(path.clone(),value.clone());
            if let Some(parent) = &self.parent{
                parent.set(path.clone(),value).await;
            }
        }

    }

    #[async_recursion]
    pub async fn delete(&self,path:String){
        self.references.lock().await.remove(&path);
        if let Some(parent) = &self.parent{
            parent.delete(path).await;
        }
    }

    pub async fn get(&self,path:String)->Option<Arc<Mutex<HeapObject>>>{
        if let Some((left,right)) = break_on(path.clone(),'.'){
            if let Some(arc) = self.references.lock().await.get(&left){
                get_value_from(right.clone(),arc.clone()).await
            } else {
                None
            }
        } else {
            if let Some(arc) = self.references.lock().await.get(&path){
                Some(arc.clone())
            } else {
                None
            }
        }


    }
}

#[async_trait]
impl IO for Context {
    async fn write(&self, data:String){
        self.user.lock().await.send(Output::new_know_that(data));
    }

    async fn read(&self, variable: Variable)->VariableValue{
        let val = if let Some(val) = self.store.get(variable.name.clone()).await{
            let ref_val = &*val.lock().await;
            if let Some(dt) = &variable.data_type {
                if ref_val.is_of_type(dt.clone()){
                    ref_val.to_value()
                } else {
                    Option::None
                }
            } else {
                ref_val.to_value()
            }
        } else {
            Option::None
        };
        if let Some(o_val)=val{
            VariableValue{
                name:variable.name.clone(),
                value:o_val.clone()
            }
        } else {
            let dt=if let Some(dt)= &variable.data_type{
                dt.clone()
            } else {
                DataType::String
            };
            self.user.lock().await.send(Output::new_tell_me(variable.name.clone(),dt.clone()));
            loop{
                let message=self.user.lock().await.get_message().await;
                if let Some(var) =match message {
                    Input::Continue(continue_input)=>continue_input.convert(),
                    _=>Option::None
                }{
                    if var.name.eq(&variable.name){
                        self.store.set(variable.name.clone(),Arc::new(Mutex::new(HeapObject::from(var.value.clone())))).await;
                        return var;
                    } else {
                        continue;
                    }
                } else {
                    self.user.lock().await.send(Output::new_know_that(format!("Invalid Value")));
                    self.user.lock().await.send(Output::new_tell_me(variable.name.clone(),dt.clone()));
                }
            }
        }

    }
}
#[derive(Clone)]
pub struct Context{
    pub user:Arc<Mutex<dyn Client>>,
    pub store:ReferenceStore,
}
impl Context {
    pub fn new(user:Arc<Mutex<dyn Client>>)->Self{
        Context{
            user:user,
            store:ReferenceStore::new()
        }
    }
    pub async fn from(context:&Context)->Self{
        Context{
            user:context.user.clone(),
            store:ReferenceStore::from(&context.store).await
        }
    }
    pub async fn delete(&self,path:String){
        self.store.delete(path).await;
    }
    pub async fn iterate<F, Fut,T>(&self,path:String,temp:String,iterate_this: F)->Vec<T>
        where
            F: FnOnce(Context) -> Fut + Copy,
            Fut: Future<Output = T>,
    {
        let mut result=vec![];
        if let Some(arc) = self.store.get(path.clone()).await{
            if let HeapObject::List(lst) = &*arc.lock().await {

                for l in lst {
                    let new_ct = Context::from(self).await;
                    new_ct.store.set(temp.clone(),l.clone()).await;
                    result.push(iterate_this(new_ct).await);
                    self.delete(temp.clone()).await;
                }
            }
        } else {
            let val=self.read(Variable{name:format!("{}::length",path.clone()),data_type:Option::Some(DataType::Long)}).await;
            let mut vec:Vec<Arc<Mutex<HeapObject>>>=vec![];
            if let Value::Long(size)=&val.value{
                for _i in 0..size.clone() {
                    let new_ct = Context::from(self).await;
                    result.push(iterate_this(new_ct).await);
                    if let Some(ho)=self.store.get(temp.clone()).await{
                        vec.push(ho.clone());
                    } else {
                        vec.push(Arc::new(Mutex::new(HeapObject::Final(Value::Null))));
                    }
                    self.delete(temp.clone()).await;
                }
                self.store.set(path.clone(),Arc::new(Mutex::new(HeapObject::List(vec)))).await
            }
        };
        result
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
pub mod tests{
    use crate::core::{Context, Client, DataType};
    use crate::core::proto::{Input, Output};
    use std::sync::{Arc, Mutex};
    use async_trait::async_trait;

    impl Context{
        pub fn mock(inputs:Vec<Input>,buffer:Arc<Mutex<Vec<Output>>>)->Self{
            let user=Arc::new(futures::lock::Mutex::new(MockClient::new(inputs,buffer)));
            Context::new(user)
        }
    }

    pub struct MockClient {
        cursur:usize,
        pub messages:Vec<Input>,
        pub buffer:Arc<Mutex<Vec<Output>>>
    }

    impl MockClient {
        pub fn new(messages:Vec<Input>,buffer:Arc<Mutex<Vec<Output>>>)->Self{
            return MockClient {
                cursur:0,
                messages,
                buffer
            };
        }
    }

    #[async_trait]
    impl Client for MockClient {
        fn send(&self, output: Output) {
            self.buffer.lock().unwrap().push(output);
        }

        async fn get_message(&mut self) -> Input {
            self.cursur = self.cursur +1;
            self.messages.get(self.cursur-1).unwrap().clone()
        }
    }

    #[tokio::test]
    async fn should_iterate(){
        let buffer= Arc::new(Mutex::new(vec![]));
        let context = Context::mock(vec![Input::new_continue("names::length".to_ascii_lowercase(), "4".to_string(), DataType::Long)],buffer.clone());
        let a=context.iterate("names".to_string(),"name".to_string(),async move |_ct|{
        }).await;
        let b=context.iterate("names".to_string(),"name".to_string(),async move |_ct|{
        }).await;
        assert_eq!(a.len(), 4);
        assert_eq!(b.len(), 4);
        assert_eq!( buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_tell_me("names::length".to_string(),DataType::Long));
    }
}
