use crate::core::{Value, DataType, Variable, VariableValue};
use std::sync::Arc;
use futures::lock::Mutex;
use std::collections::HashMap;
use async_recursion::async_recursion;
use async_trait::async_trait;
use crate::core::proto::{Input, Output};
use std::future::Future;
use crate::journey::Journey;
use crate::template::VariableReferenceName;

pub enum HeapObject{
    Final(Value),
    List(Vec<Arc<Mutex<HeapObject>>>),
    Object(HashMap<String,Arc<Mutex<HeapObject>>>),
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
                    DataType::Object=>true,
                    _=>false,
                }
            }
        }
    }
    #[async_recursion]
    pub async fn to_value(&self)->Value{
        match self {
            HeapObject::Final(val)=>{
                val.clone()
            },
            HeapObject::List(lst)=>{
                let mut vec_val=Vec::new();
                for val in lst {
                    vec_val.push(val.lock().await.to_value().await)
                }
                return Value::Array(vec_val);
            },
            HeapObject::Object(obj)=>{
                let mut hm_val=HashMap::new();
                for (key,val) in obj {
                    hm_val.insert(key.clone(),val.lock().await.to_value().await);
                }
                return Value::Map(hm_val);
            }
        }
    }
}
#[derive(Clone)]
pub struct ConnectionStore{
    parent:Option<Box<ConnectionStore>>,
    references:Arc<Mutex<HashMap<String,Arc<Mutex<Box<dyn rdbc_async::sql::Connection>>>>>>
}
#[derive(Debug, Clone)]
pub struct ReferenceStore{
    parent:Option<Box<ReferenceStore>>,
    references:Arc<Mutex<HashMap<String,Arc<Mutex<HeapObject>>>>>
}

pub fn break_on(path:String,chr:char)->Option<(String,String)>{
    let spl:Vec<&str>=path.splitn(2,chr).collect();
    if spl.len() == 2{
        Option::Some((spl[0].to_string(),spl[1].to_string()))
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
                    let inner_obj = Arc::new(Mutex::new(HeapObject::Object(HashMap::new())));
                    obj.insert(left.clone(),inner_obj.clone());
                    set_value_at(right.clone(),inner_obj.clone(),value).await
                }
            } else {
                obj.insert(path.clone(),value.clone());
                Option::Some(value)
            }
        },
        _=> Option::None
    }
}
impl ConnectionStore{
    pub fn new()->Self{
        Self{
            parent:Option::None,
            references:Arc::new(Mutex::new(HashMap::new()))
        }
    }
    pub async fn from(rs:&ConnectionStore)->Self{
        return Self{
            parent:Option::Some(Box::new(rs.clone())),
            references:Arc::new(Mutex::new(rs.references.lock().await.clone()))
        }
    }

    pub async fn get(&self,path:VariableReferenceName)->Option<Arc<Mutex<Box<dyn rdbc_async::sql::Connection>>>>{
        let tmp = self.references.lock().await;
        tmp.get(&(path.to_string())).map(|arc|arc.clone())
    }
    pub async fn define(&self,path:String,connection:Box<dyn rdbc_async::sql::Connection>){
        let mut refs = self.references.lock().await;
        refs.insert(path,Arc::new(Mutex::new(connection)));
    }
    pub async fn undefine(&self,path:String){
        let mut refs = self.references.lock().await;
        refs.remove(&path);
    }
}
impl ReferenceStore{
    pub fn new()->Self{
        Self{
            parent:Option::None,
            references:Arc::new(Mutex::new(HashMap::new()))
        }
    }
    pub async fn from(rs:&ReferenceStore)->Self{
        return Self{
            parent:Option::Some(Box::new(rs.clone())),
            references:Arc::new(Mutex::new(rs.references.lock().await.clone()))
        }
    }
    #[async_recursion]
    pub async fn push(&self,path:String,value:Arc<Mutex<HeapObject>>){
        let mut new_array = false;
        let array = if let Some(arc) = self.references.lock().await.get(&path){
            arc.clone()
        } else {
            new_array = true;
            Arc::new(Mutex::new(HeapObject::List(vec![value.clone()])))
        };
        if new_array {
            self.references.lock().await.insert(path.clone(),array.clone());
            if let Some(parent) = &self.parent{
                parent.set(path.clone(),array.clone()).await;
            }
        } else {
            let ho =&mut *array.lock().await;
            match ho {
                HeapObject::List(ls)=>{
                    ls.push(value);
                }
                _=>{}
            }
        }
    }
    #[async_recursion]
    pub async fn set(&self,path:String,value:Arc<Mutex<HeapObject>>){
        if let Some((left,right)) = break_on(path.clone(),'.'){
            let mut new_obj = false;
            let obj = if let Some(arc) = self.references.lock().await.get(&left){
                arc.clone()
            } else {
                new_obj=true;
                Arc::new(Mutex::new(HeapObject::Object(HashMap::new())))
            };
            set_value_at(right.clone(),obj.clone(),value).await;
            if new_obj {
                self.references.lock().await.insert(left.clone(),obj.clone());
                if let Some(parent) = &self.parent{
                    parent.set(left.clone(),obj.clone()).await;
                }
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
        self.user.lock().await.send(Output::new_know_that(data)).await;
    }

    async fn read(&self, variable: Variable)->VariableValue{
        let val = if let Some(val) = self.store.get(variable.name.clone()).await{
            let ref_val = &*val.lock().await;
            if let Some(dt) = &variable.data_type {
                if ref_val.is_of_type(dt.clone()){
                    Option::Some(ref_val.to_value().await)
                } else {
                    Option::None
                }
            } else {
                Option::Some(ref_val.to_value().await)
            }
        } else {
            Option::None
        };
        if let Some(o_val)=val{
            VariableValue{
                name:variable.name.clone(),
                value:o_val.clone()
            }
        } else if self.fallback {
            let dt=if let Some(dt)= &variable.data_type{
                dt.clone()
            } else {
                DataType::String
            };
            self.user.lock().await.send(Output::new_tell_me(variable.name.clone(),dt.clone())).await;
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
                    self.user.lock().await.send(Output::new_know_that(format!("Invalid Value"))).await;
                    self.user.lock().await.send(Output::new_tell_me(variable.name.clone(),dt.clone())).await;
                }
            }
        } else {
            VariableValue{
                name:variable.name.clone(),
                value:Value::Null
            }
        }

    }
}
#[derive(Clone)]
pub struct Context{
    pub journeys:Vec<Journey>,
    pub user:Arc<Mutex<dyn Client>>,
    pub store:ReferenceStore,
    pub connection_store:ConnectionStore,
    pub fallback:bool
}
impl Context {
    pub async fn get_var_from_store(&self,name:String)->Option<Value>{
        if let Some(var)=self.store.get(name).await{
            Option::Some(var.lock().await.to_value().await)
        } else {
            Option::None
        }
    }
    pub fn new(user:Arc<Mutex<dyn Client>>,journeys:Vec<Journey>)->Self{
        Context{
            journeys,
            user:user,
            connection_store:ConnectionStore::new(),
            store:ReferenceStore::new(),
            fallback:true
        }
    }
    pub async fn define(&self,var:String,value:Value){
        self.store.set(var,Arc::new(Mutex::new(value.to_heap_object()))).await;
    }
    pub async fn push(&self,var:String,value:Value){
        self.store.push(var,Arc::new(Mutex::new(value.to_heap_object()))).await;
    }
    pub async fn from(context:&Context)->Self{
        Context{
            journeys:context.journeys.clone(),
            user:context.user.clone(),
            connection_store:ConnectionStore::from(&context.connection_store).await,
            store:ReferenceStore::from(&context.store).await,
            fallback:context.fallback
        }
    }
    pub async fn from_without_fallback(context:&Context)->Self{
        Context{
            journeys:context.journeys.clone(),
            user:context.user.clone(),
            connection_store:ConnectionStore::from(&context.connection_store).await,
            store:ReferenceStore::from(&context.store).await,
            fallback:false
        }
    }
    pub async fn delete(&self,path:String){
        self.store.delete(path).await;
    }
    pub async fn iterate_like<F, Fut,T,U>(&self, vec_val:Vec<U>, path:String, temp:String, iterate_this: F) ->Vec<T>
        where
            F: FnOnce(Context,usize,U) -> Fut + Copy,
            Fut: Future<Output = T>,
    {
        let mut result=vec![];

        let mut vec:Vec<Arc<Mutex<HeapObject>>>=vec![];
        let mut i =0;
        for value in vec_val {
            let new_ct = Context::from(self).await;
            result.push(iterate_this(new_ct,i,value).await);
            if let Some(ho)=self.store.get(temp.clone()).await{
                vec.push(ho.clone());
            } else {
                vec.push(Arc::new(Mutex::new(HeapObject::Final(Value::Null))));
            }
            self.delete(temp.clone()).await;
            i = i + 1;
        }
        self.store.set(path.clone(),Arc::new(Mutex::new(HeapObject::List(vec)))).await;
        result
    }
    pub async fn iterate<F, Fut,T>(&self,path:String,opt_temp:Option<String>,iterate_this: F)->Vec<T>
        where
            F: FnOnce(Context,usize) -> Fut + Copy,
            Fut: Future<Output = T>,
    {
        let mut result=vec![];

        if let Some(arc) = self.store.get(path.clone()).await{

            if let HeapObject::List(lst) = &*arc.lock().await {
                let mut i = 0;

                for l in lst {
                    let new_ct = Context::from(self).await;
                    if let Some(temp) = opt_temp.clone() {
                        new_ct.store.set(temp.clone(),l.clone()).await;
                    }
                    result.push(iterate_this(new_ct,i).await);
                    if let Some(temp) = opt_temp.clone() {
                        self.delete(temp.clone()).await;
                    }
                    i = i + 1;
                }
            }
        } else {
            let val=self.read(Variable{name:format!("{}::length",path.clone()),data_type:Option::Some(DataType::PositiveInteger)}).await;
            let mut vec:Vec<Arc<Mutex<HeapObject>>>=vec![];
            if let Value::PositiveInteger(size)=&val.value{
                for i in 0..size.clone() {
                    let new_ct = Context::from(self).await;
                    result.push(iterate_this(new_ct,i as usize).await);
                    if let Some(temp) = opt_temp.clone() {
                        if let Some(ho) = self.store.get(temp.clone()).await {
                            vec.push(ho.clone());
                        } else {
                            vec.push(Arc::new(Mutex::new(HeapObject::Final(Value::Null))));
                        }
                        self.delete(temp.clone()).await;
                    } else {
                        vec.push(Arc::new(Mutex::new(HeapObject::Final(Value::Null))));
                    }
                }
                self.store.set(path.clone(),Arc::new(Mutex::new(HeapObject::List(vec)))).await
            }
        };
        result
    }
}
#[async_trait]
pub trait Client:Send+Sync{
    async fn send(&self,output:Output);
    async fn get_message(&mut self)->Input;
}
#[async_trait]
pub trait IO {
    async fn write(&self,data:String);
    async fn read(&self,variable:Variable)->VariableValue;
}
pub struct MockClient {
    cursur:usize,
    pub messages:Vec<Input>,
    pub buffer:Arc<std::sync::Mutex<Vec<Output>>>
}

impl Context{
    pub fn mock(inputs:Vec<Input>,buffer:Arc<std::sync::Mutex<Vec<Output>>>)->Self{
        let user=Arc::new(futures::lock::Mutex::new(MockClient::new(inputs,buffer)));
        Context::new(user,vec![])
    }
}

impl MockClient {
    pub fn new(messages:Vec<Input>,buffer:Arc<std::sync::Mutex<Vec<Output>>>)->Self{
        return MockClient {
            cursur:0,
            messages,
            buffer
        };
    }
}

#[async_trait]
impl Client for MockClient {
    async fn send(&self, output: Output) {
        self.buffer.lock().unwrap().push(output);
    }

    async fn get_message(&mut self) -> Input {
        self.cursur = self.cursur +1;
        self.messages.get(self.cursur-1).unwrap().clone()
    }
}

#[cfg(test)]
pub mod tests{
    use crate::core::{DataType, Variable};
    use crate::core::proto::{Input, Output};
    use std::sync::{Arc, Mutex};
    use async_trait::async_trait;
    use crate::core::runtime::{Context, Client, IO, break_on};




    #[tokio::test]
    async fn should_iterate(){
        let buffer= Arc::new(Mutex::new(vec![]));
        let context = Context::mock(vec![Input::new_continue("names::length".to_ascii_lowercase(), "4".to_string(), DataType::PositiveInteger)],buffer.clone());
        let a=context.iterate("names".to_string(),Option::Some("name".to_string()),async move |_ct,_i|{
        }).await;
        let b=context.iterate("names".to_string(),Option::Some("name".to_string()),async move |_ct,_i|{
        }).await;
        assert_eq!(a.len(), 4);
        assert_eq!(b.len(), 4);
        assert_eq!( buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_tell_me("names::length".to_string(),DataType::PositiveInteger));
    }
    #[tokio::test]
    async fn should_break_on_dot_with_two_dots(){
        let vals= break_on(format!("person.address.addressLine1"),'.');
        let (left,right) = vals.unwrap();

        assert_eq!(left,format!("person"));
        assert_eq!(right,format!("address.addressLine1"));

    }
    #[tokio::test]
    async fn should_iterate_and_read(){
        let buffer= Arc::new(Mutex::new(vec![]));
        let context = Context::mock(vec![
            Input::new_continue("names::length".to_ascii_lowercase(), "2".to_string(), DataType::PositiveInteger),
            Input::new_continue("name".to_ascii_lowercase(), "Atmaram".to_string(), DataType::String),
            Input::new_continue("name".to_ascii_lowercase(), "Atiksh".to_string(), DataType::String)
        ],buffer.clone());
        let a=context.iterate("names".to_string(),Option::Some("name".to_string()),async move |ct,_|{
            ct.read(Variable{
                name:"name".to_string(),
                data_type:Option::Some(DataType::String)
            }).await
        }).await;
        let b=context.iterate("names".to_string(),Option::Some("name".to_string()),async move |ct,_|{
            ct.read(Variable{
                name:"name".to_string(),
                data_type:Option::Some(DataType::String)
            }).await
        }).await;
        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 2);
        assert_eq!( buffer.lock().unwrap().len(),3);
        assert_eq!( buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_tell_me("names::length".to_string(),DataType::PositiveInteger));
        assert_eq!( buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_tell_me("name".to_string(),DataType::String));
        assert_eq!( buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_tell_me("name".to_string(),DataType::String));
    }

    #[tokio::test]
    async fn should_iterate_and_read_internal_variables(){
        let buffer= Arc::new(Mutex::new(vec![]));
        let context = Context::mock(vec![
            Input::new_continue("persons::length".to_ascii_lowercase(), "2".to_string(), DataType::PositiveInteger),
            Input::new_continue("person.name".to_ascii_lowercase(), "Atmaram".to_string(), DataType::String),
            Input::new_continue("person.name".to_ascii_lowercase(), "Atiksh".to_string(), DataType::String)
        ],buffer.clone());
        let a=context.iterate("persons".to_string(),Option::Some("person".to_string()),async move |ct,_|{
            ct.read(Variable{
                name:"person.name".to_string(),
                data_type:Option::Some(DataType::String)
            }).await
        }).await;
        let b=context.iterate("persons".to_string(),Option::Some("person".to_string()),async move |ct,_|{
            ct.read(Variable{
                name:"person.name".to_string(),
                data_type:Option::Some(DataType::String)
            }).await
        }).await;
        assert_eq!(a.len(), 2);
        assert_eq!(b.len(), 2);
        assert_eq!( buffer.lock().unwrap().len(),3);
        assert_eq!( buffer.lock().unwrap().get(0).unwrap().clone(),Output::new_tell_me("persons::length".to_string(),DataType::PositiveInteger));
        assert_eq!( buffer.lock().unwrap().get(1).unwrap().clone(),Output::new_tell_me("person.name".to_string(),DataType::String));
        assert_eq!( buffer.lock().unwrap().get(2).unwrap().clone(),Output::new_tell_me("person.name".to_string(),DataType::String));
    }
}