use crate::core::{Value, DataType, Variable, VariableValue};
use std::sync::{Arc};
use tokio::sync::RwLock;
use futures::lock::Mutex;
use std::collections::HashMap;
use async_recursion::async_recursion;
use async_trait::async_trait;
use crate::core::proto::{Input, Output};
use std::future::Future;
use futures_util::stream::SplitSink;
use num_traits::ToPrimitive;
use test::stats::Stats;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::core::scrapper::none::NoneScraper;
use crate::core::scrapper::Scrapper;
use crate::journey::Journey;
use crate::template::rest::RestVerb;
use crate::template::VariableReferenceName;
pub enum HeapObject{
    Final(Value),
    List(Vec<Arc<RwLock<HeapObject>>>),
    Object(HashMap<String,Arc<RwLock<HeapObject>>>),
}

impl HeapObject{
    #[async_recursion]
    pub async fn copy(&self)->HeapObject {
        match &self {
            HeapObject::Final(val)=>{
                HeapObject::Final(val.clone())
            },
            HeapObject::List(vals)=>{
                let mut list = vec![];
                for val in vals {
                    let obj = val.read().await;
                    list.push(Arc::new(RwLock::new((*obj).copy().await)))
                }
                HeapObject::List(list)
            },
            HeapObject::Object(obj)=>{
                let mut map = HashMap::new();
                for (key,value) in obj{
                    let arc = value.clone();
                    let val = arc.read().await;
                    map.insert(key.clone(),Arc::new(RwLock::new((*val).copy().await)));
                }
                HeapObject::Object(map)
            }
        }
    }
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
                    vec_val.push(val.read().await.to_value().await)
                }
                return Value::Array(vec_val);
            },
            HeapObject::Object(obj)=>{
                let mut hm_val=HashMap::new();
                for (key,val) in obj {
                    hm_val.insert(key.clone(),val.read().await.to_value().await);
                }
                return Value::Map(hm_val);
            }
        }
    }
}
#[derive(Clone)]
pub struct RestStatsStore{
    parent:Option<Box<RestStatsStore>>,
    samples:Arc<Mutex<Vec<(RestVerb,String,u128)>>>
}
#[derive(Clone)]
pub struct TransactionsStatsStore{
    parent:Option<Box<TransactionsStatsStore>>,
    samples:Arc<Mutex<Vec<(String,u128)>>>
}
#[derive(Clone)]
pub struct WebsocketConnectionStore{
    references:Arc<Mutex<HashMap<String,Arc<Mutex<SplitSink<WebSocketStream<tokio_tungstenite::stream::Stream<tokio::net::TcpStream,hyper_tls::TlsStream<tokio::net::TcpStream>>>,Message>>>>>>
}
#[derive(Clone)]
pub struct ConnectionStore{
    references:Arc<Mutex<HashMap<String,Arc<Mutex<Box<dyn rdbc_async::sql::Connection>>>>>>
}
#[derive(Clone)]
pub struct ReferenceStore{
    parent:Option<Box<ReferenceStore>>,
    references:Arc<RwLock<HashMap<String,Arc<RwLock<HeapObject>>>>>
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
pub async fn contains_reference_in(path:String,heap_object_ref:Arc<RwLock<HeapObject>>)->bool{
    let ho = &*heap_object_ref.read().await;
    match ho {
        HeapObject::Object(obj)=>{
            if let Some((left,right))=break_on(path.clone(),'.'){
                if let Some(key_value)=obj.get(&left){
                    contains_reference_in(right.clone(),key_value.clone()).await
                } else {
                    false
                }

            } else {
                if let Some(_)=obj.get(&path.clone()){
                    true
                } else {
                    false
                }
            }
        },
        _=> false
    }
}

#[async_recursion]
pub async fn get_value_from(path:String,heap_object_ref:Arc<RwLock<HeapObject>>)->Option<Arc<RwLock<HeapObject>>>{
    let ho = &*heap_object_ref.read().await;
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
pub async fn set_value_at(path:String,heap_object_ref:Arc<RwLock<HeapObject>>,value:Arc<RwLock<HeapObject>>)->Option<Arc<RwLock<HeapObject>>>{
    let ho = &mut *heap_object_ref.write().await;
    match ho {
        HeapObject::Object(obj)=>{
            if let Some((left,right))=break_on(path.clone(),'.'){
                if let Some(key_value)=obj.get(&left){
                    set_value_at(right.clone(),key_value.clone(),value).await
                } else {
                    let inner_obj = Arc::new(RwLock::new(HeapObject::Object(HashMap::new())));
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
impl RestStatsStore{
    pub fn new()->Self{
        Self{
            parent:Option::None,
            samples:Arc::new(Mutex::new(vec![]))
        }
    }
    pub async fn print_stats(&self){
        let samples = self.samples.lock().await;
        for (v,u,t) in &(*samples){
            println!("{:?}-{}=>{}",v,u,t)
        }
    }
    pub async fn print_stats_summary(&self){
        let samples = self.samples.lock().await;
        if (&*samples).len() > 0 {
            let samples:Vec<f64> =(&(*samples)).iter().map(|(_v,_u,t)|t.to_f64().unwrap()).collect();
            println!("MIN: {}",samples.min());
            println!("MAX: {}",samples.max());
            println!("Average: {}",samples.mean());
        }
    }
    pub async fn get_stats(&self)->Vec<(RestVerb,String,u128)>{
        let samples = self.samples.lock().await;
        (&(*samples)).iter().map(|(v,u,t)|(v.clone(),u.clone(),t.clone())).collect()
    }
    pub async fn from(rs:&RestStatsStore)->Self{
        return Self{
            parent:Option::Some(Box::new(rs.clone())),
            samples:Arc::new(Mutex::new(rs.samples.lock().await.clone()))
        }
    }

    #[async_recursion]
    pub async fn push_stat(&self,stat:(RestVerb,String,u128)){
        let mut refs = self.samples.lock().await;
        refs.push(stat.clone());
        if let Some(p)=&self.parent{
            p.push_stat(stat).await;
        }
    }

}
pub fn group_by(samples:Vec<(String,f64)>) -> Vec<(String, Vec<f64>)>
{
    let mut map:HashMap<String,Vec<f64>> = HashMap::new();
    for (key,value) in samples {
        if map.contains_key(&key) {
            if let Some(ref mut v) = map.get_mut(&key) {
                v.push(value)
            }
        } else {
            map.insert(key,vec![value]);
        }
    }
    map.iter().map(|(key,val)|(key.clone(),val.clone())).collect()
}
impl TransactionsStatsStore{
    pub fn new()->Self{
        Self{
            parent:Option::None,
            samples:Arc::new(Mutex::new(vec![]))
        }
    }
    pub async fn print_stats(&self){
        let samples = self.samples.lock().await;
        for (tr,tm) in &(*samples){
            println!("{}=>{}",tr,tm)
        }
    }
    pub async fn print_stats_summary(&self){
        let samples = self.samples.lock().await;
        if (&*samples).len()>0 {
            let samples:Vec<(String,f64)> =(&(*samples)).iter().map(|(u,t)|(u.clone(),t.to_f64().unwrap())).collect();
            let groups = group_by(samples);
            println!("{:30}{:>20}{:>20}{:>20}{:>20}{:>20}{:>20}","Transaction","Min","Max","Average","90%","95%","Total Samples");
            for (tr,sam) in groups{
                let samp:Vec<f64> = sam.iter().map(|tm|tm.clone()).collect();
                println!("{:30}{:20.2}{:20.2}{:20.2}{:20.2}{:20.2}{:20}",tr,samp.min(),samp.max(),samp.mean(),samp.percentile(90.0),samp.percentile(95.0,),samp.len());

            }
        }


    }
    pub async fn get_stats(&self)->Vec<(String,u128)>{
        let samples = self.samples.lock().await;
        (&(*samples)).iter().map(|(u,t)|(u.clone(),t.clone())).collect()
    }
    pub async fn from(rs:&TransactionsStatsStore)->Self{
        return Self{
            parent:Option::Some(Box::new(rs.clone())),
            samples:Arc::new(Mutex::new(rs.samples.lock().await.clone()))
        }
    }

    #[async_recursion]
    pub async fn push_stat(&self,stat:(String,u128)){
        let mut refs = self.samples.lock().await;
        refs.push(stat.clone());
        if let Some(p)=&self.parent{
            p.push_stat(stat).await;
        }
    }

}
impl ConnectionStore{
    pub fn new()->Self{
        Self{
            references:Arc::new(Mutex::new(HashMap::new()))
        }
    }
    pub async fn from(rs:&ConnectionStore)->Self{
        return Self{
            references:rs.references.clone()
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
impl WebsocketConnectionStore{
    pub fn new()->Self{
        Self{
            references:Arc::new(Mutex::new(HashMap::new()))
        }
    }
    pub async fn from(rs:&WebsocketConnectionStore)->Self{
        return Self{
            references:rs.references.clone()
        }
    }

    pub async fn get(&self,name:String)->Option<Arc<Mutex<SplitSink<WebSocketStream<tokio_tungstenite::stream::Stream<tokio::net::TcpStream,hyper_tls::TlsStream<tokio::net::TcpStream>>>,Message>>>>{
        let tmp = self.references.lock().await;
        tmp.get(&(name)).map(|arc|arc.clone())
    }
    pub async fn define(&self,path:String,connection:SplitSink<WebSocketStream<tokio_tungstenite::stream::Stream<tokio::net::TcpStream,hyper_tls::TlsStream<tokio::net::TcpStream>>>,Message>){
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
            references:Arc::new(RwLock::new(HashMap::new()))
        }
    }
    pub async fn new_from_references(references:Arc<RwLock<HashMap<String,Arc<RwLock<HeapObject>>>>>)->Self{
        let mut hm = HashMap::new();
        let refs = references.read().await;
        for (key,value) in &(*refs) {
            let obj = value.read().await;
            hm.insert(key.clone(),Arc::new(RwLock::new((*obj).copy().await)));
        }
        ReferenceStore{
            references:Arc::new(RwLock::new(hm)),
            parent:None
        }
    }
    pub async fn from(rs:&ReferenceStore)->Self{
        return Self{
            parent:Option::Some(Box::new(rs.clone())),
            references:Arc::new(RwLock::new(rs.references.read().await.clone()))
        }
    }
    #[async_recursion]
    pub async fn push(&self,path:String,value:Arc<RwLock<HeapObject>>){
        let mut new_array = false;
        let array = if let Some(arc) = self.references.read().await.get(&path){
            arc.clone()
        } else {
            new_array = true;
            Arc::new(RwLock::new(HeapObject::List(vec![value.clone()])))
        };
        if new_array {
            self.references.write().await.insert(path.clone(),array.clone());
            if let Some(parent) = &self.parent{
                parent.set(path.clone(),array.clone()).await;
            }
        } else {
            let ho =&mut *array.write().await;
            match ho {
                HeapObject::List(ls)=>{
                    ls.push(value);
                }
                _=>{}
            }
        }
    }
    #[async_recursion]
    pub async fn set(&self,path:String,value:Arc<RwLock<HeapObject>>){
        if let Some((left,right)) = break_on(path.clone(),'.'){
            let mut new_obj = false;
            let obj = if let Some(arc) = self.references.read().await.get(&left){
                arc.clone()
            } else {
                new_obj=true;
                Arc::new(RwLock::new(HeapObject::Object(HashMap::new())))
            };
            set_value_at(right.clone(),obj.clone(),value).await;
            if new_obj {
                self.references.write().await.insert(left.clone(),obj.clone());
                if let Some(parent) = &self.parent{
                    parent.set(left.clone(),obj.clone()).await;
                }
            }
        } else {
            self.references.write().await.insert(path.clone(),value.clone());
            if let Some(parent) = &self.parent{
                parent.set(path.clone(),value).await;
            }
        }

    }

    #[async_recursion]
    pub async fn delete(&self,path:String){
        self.references.write().await.remove(&path);
        if let Some(parent) = &self.parent{
            parent.delete(path).await;
        }
    }
    pub async fn has_parent(&self,path:String)->bool{
        if let Some((left,_)) = break_on(path.clone(),'.'){
            if let Some(_) = self.references.read().await.get(&left){
                true
            } else {
                false
            }
        } else {
            if let Some(_) = self.references.read().await.get(&path){
                true
            } else {
                false
            }
        }
    }
    pub async fn contains_reference(&self,path:String)->bool{
        if let Some((left,right)) = break_on(path.clone(),'.'){
            if let Some(arc) = self.references.read().await.get(&left){
                contains_reference_in(right.clone(),arc.clone()).await
            } else {
                false
            }
        } else {
            if let Some(_) = self.references.read().await.get(&path){
                true
            } else {
                false
            }
        }
    }
    pub async fn get(&self,path:String)->Option<Arc<RwLock<HeapObject>>>{
        if let Some((left,right)) = break_on(path.clone(),'.'){
            if let Some(arc) = self.references.read().await.get(&left){
                get_value_from(right.clone(),arc.clone()).await
            } else {
                None
            }
        } else {
            if let Some(arc) = self.references.read().await.get(&path){
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
            let ref_val = &*val.read().await;
            if let Some(dt) = &variable.data_type {
                if ref_val.is_of_type(dt.clone()){
                    Option::Some(ref_val.to_value().await)
                } else {
                    Option::None
                }
            } else {
                Option::Some(ref_val.to_value().await)
            }
        } else if let Some(val) = self.global_store.get(variable.name.clone()).await{
            let ref_val = &*val.read().await;
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
            None
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
                        self.set(variable.name.clone(),Arc::new(RwLock::new(HeapObject::from(var.value.clone())))).await;
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
    pub debug:bool,
    pub scrapper:Arc<Box<dyn Scrapper>>,
    pub local_program_lookup:Arc<RwLock<HashMap<String,Arc<Journey>>>>,
    pub global_program_lookup:HashMap<String,Arc<Journey>>,
    pub user:Arc<Mutex<dyn Client>>,
    global_store:ReferenceStore,
    store:ReferenceStore,
    pub connection_store:ConnectionStore,
    pub websocket_connection_store:WebsocketConnectionStore,
    pub rest_stats_store:RestStatsStore,
    pub tr_stats_store:TransactionsStatsStore,
    pub fallback:bool,
    pub sender:Option<Arc<Mutex<tokio::sync::mpsc::UnboundedSender<i32>>>>
}
impl Context {
    pub async fn get_local_journey(&self,name:String)->Option<Arc<Journey>>{
        self.local_program_lookup.read().await.get(&name).map(|a|a.clone())
    }
    pub fn get_global_journey(&self,name:String)->Option<Arc<Journey>>{
        self.global_program_lookup.get(&name).map(|a|a.clone())
    }
    pub fn exiter(&mut self)->tokio::sync::mpsc::UnboundedReceiver<i32>{
        let (tx,rx) = tokio::sync::mpsc::unbounded_channel::<i32>();
        self.sender = Some(Arc::new(Mutex::new(tx)));
        rx
    }
    pub async fn exit(&self,message:i32){
        if let Some(tx) = self.sender.clone() {
            let vl = tx.lock().await;
            let _ = (*vl).send(message);
        }
    }
    pub async fn get_var_from_store(&self,name:String)->Option<Value>{
        if let Some(var)=self.store.get(name.clone()).await{
            Option::Some(var.read().await.to_value().await)
        } else {
            if let Some(var)=self.global_store.get(name).await{
                Option::Some(var.read().await.to_value().await)
            } else {
                Option::None
            }
        }
    }
    pub async fn copy_from(context:&Context)->Context{
        Context{
            debug:context.debug,
            sender:Option::None,
            scrapper:context.scrapper.clone(),
            local_program_lookup:context.local_program_lookup.clone(),
            global_program_lookup:context.global_program_lookup.clone(),
            user:context.user.clone(),
            connection_store:ConnectionStore::new(),
            websocket_connection_store:WebsocketConnectionStore::new(),
            rest_stats_store:context.rest_stats_store.clone(),
            tr_stats_store:context.tr_stats_store.clone(),
            global_store:context.global_store.clone(),
            store:ReferenceStore::new_from_references(context.store.references.clone()).await,
            fallback:context.fallback
        }
    }
    pub fn new(user:Arc<Mutex<dyn Client>>,journeys:HashMap<String,Arc<Journey>>,scrapper:Arc<Box<dyn Scrapper>>,debug:bool)->Self{
        Context{
            debug,
            sender:Option::None,
            scrapper,
            local_program_lookup:Arc::new(RwLock::new(HashMap::new())),
            global_program_lookup: journeys,
            user:user,
            connection_store:ConnectionStore::new(),
            websocket_connection_store:WebsocketConnectionStore::new(),
            rest_stats_store:RestStatsStore::new(),
            tr_stats_store:TransactionsStatsStore::new(),
            global_store:ReferenceStore::new(),
            store:ReferenceStore::new(),
            fallback:true
        }
    }
    pub async fn define(&self,var:String,value:Value){
        self.store.set(var,Arc::new(RwLock::new(value.to_heap_object()))).await;
    }
    pub async fn global_define(&self,var:String,value:Value){
        self.global_store.set(var,Arc::new(RwLock::new(value.to_heap_object()))).await;
    }
    pub async fn undefine(&self,var:String){
        self.delete(var).await;
    }
    pub async fn push(&self,var:String,value:Value){
        if self.store.contains_reference(var.clone()).await{
            self.store.push(var,Arc::new(RwLock::new(value.to_heap_object()))).await;
        } else if self.global_store.contains_reference(var.clone()).await {
            self.global_store.push(var,Arc::new(RwLock::new(value.to_heap_object()))).await;
        } else {
            self.store.push(var,Arc::new(RwLock::new(value.to_heap_object()))).await;
        }

    }
    pub async fn set(&self,path:String,value:Arc<RwLock<HeapObject>>){
        if self.store.has_parent(path.clone()).await {
            self.store.set(path,value).await;
        } else if self.global_store.has_parent(path.clone()).await {
            self.global_store.set(path,value).await;
        } else {
            self.store.set(path,value).await;
        }
    }
    pub async fn from(context:&Context)->Self{
        Context{
            debug:context.debug,
            sender:context.sender.clone(),
            scrapper:context.scrapper.clone(),
            local_program_lookup:context.local_program_lookup.clone(),
            global_program_lookup:context.global_program_lookup.clone(),
            user:context.user.clone(),
            connection_store:ConnectionStore::from(&context.connection_store).await,
            websocket_connection_store:WebsocketConnectionStore::from(&context.websocket_connection_store).await,
            rest_stats_store:RestStatsStore::from(&context.rest_stats_store).await,
            tr_stats_store:TransactionsStatsStore::from(&context.tr_stats_store).await,
            global_store:context.global_store.clone(),
            store:ReferenceStore::from(&context.store).await,
            fallback:context.fallback
        }
    }
    pub async fn from_without_fallback(context:&Context)->Self{
        Context{
            debug:context.debug,
            sender:context.sender.clone(),
            scrapper: context.scrapper.clone(),
            local_program_lookup:context.local_program_lookup.clone(),
            global_program_lookup:context.global_program_lookup.clone(),
            user:context.user.clone(),
            connection_store:ConnectionStore::from(&context.connection_store).await,
            websocket_connection_store:WebsocketConnectionStore::from(&context.websocket_connection_store).await,
            rest_stats_store:RestStatsStore::from(&context.rest_stats_store).await,
            tr_stats_store:TransactionsStatsStore::from(&context.tr_stats_store).await,
            global_store:context.global_store.clone(),
            store:ReferenceStore::from(&context.store).await,
            fallback:false
        }
    }
    pub async fn delete(&self,path:String){
        if self.store.contains_reference(path.clone()).await{
            self.store.delete(path).await;
        } else if self.global_store.contains_reference(path.clone()).await{
            self.global_store.delete(path).await
        } else {
            self.store.delete(path).await;
        }
    }
    pub async fn iterate_like<F, Fut,T,U>(&self, vec_val:Vec<U>, path:String, temp:String, iterate_this: F) ->Vec<T>
        where
            F: FnOnce(Context,usize,U) -> Fut + Copy,
            Fut: Future<Output = T>,
    {
        let mut result=vec![];

        let mut vec:Vec<Arc<RwLock<HeapObject>>>=vec![];
        let mut i =0;
        for value in vec_val {
            let new_ct = Context::from(self).await;
            result.push(iterate_this(new_ct,i,value).await);
            if let Some(ho)=self.store.get(temp.clone()).await{
                vec.push(ho.clone());
            } else {
                vec.push(Arc::new(RwLock::new(HeapObject::Final(Value::Null))));
            }
            self.delete(temp.clone()).await;
            i = i + 1;
        }
        self.set(path.clone(),Arc::new(RwLock::new(HeapObject::List(vec)))).await;
        result
    }
    pub async fn iterate<F, Fut,T>(&self,path:String,opt_temp:Option<String>,iterate_this: F)->Vec<T>
        where
            F: FnOnce(Context,usize) -> Fut + Copy,
            Fut: Future<Output = T>,
    {
        let mut result=vec![];

        if let Some(arc) = self.store.get(path.clone()).await{

            if let HeapObject::List(lst) = &*arc.read().await {
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
            let mut vec:Vec<Arc<RwLock<HeapObject>>>=vec![];
            if let Value::PositiveInteger(size)=&val.value{
                for i in 0..size.clone() {
                    let new_ct = Context::from(self).await;
                    result.push(iterate_this(new_ct,i as usize).await);
                    if let Some(temp) = opt_temp.clone() {
                        if let Some(ho) = self.store.get(temp.clone()).await {
                            vec.push(ho.clone());
                        } else {
                            vec.push(Arc::new(RwLock::new(HeapObject::Final(Value::Null))));
                        }
                        self.delete(temp.clone()).await;
                    } else {
                        vec.push(Arc::new(RwLock::new(HeapObject::Final(Value::Null))));
                    }
                }
                self.set(path.clone(),Arc::new(RwLock::new(HeapObject::List(vec)))).await
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
        Context::new(user,HashMap::new(),Arc::new(Box::new(NoneScraper{})),false)
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
    
    use crate::core::runtime::{Context, IO, break_on};




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