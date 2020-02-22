use crate::io::StringIO;
use serde::{Serialize, Deserialize, Serializer};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Debug;
use std::result::Result;
use crate::break_on;
use std::borrow::BorrowMut;

pub struct Messanger<T> where T:StringIO{
    pub string_io:Box<T>
}
impl<T> Messanger<T> where T:StringIO{
    pub fn new(str_io:T)->Messanger<T>{
        Messanger{
            string_io:Box::new(str_io)
        }
    }
    pub fn ask(&mut self, var_desc: VariableDesciption)->RawVariableValue {
        self.string_io.write(format!("Please enter value for {} of type {:?}",var_desc.name,var_desc.data_type));
        let line = self.string_io.read();
        RawVariableValue {
            value:Option::Some(line),
            name:var_desc.name.clone(),
            data_type:var_desc.data_type.clone()
        }

    }

    pub fn tell(&mut self, words: String) {
        self.string_io.write(words);
    }
}

#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub struct VariableDesciption {
    pub name:String,
    pub data_type:VarType
}

#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub enum VarType{
    String,
    Long,
    Boolean,
    Double,
    Object,
    List,
}
#[derive(Debug,PartialEq,Clone)]
pub enum Value{
    String(String),
    Long(i64),
    Boolean(bool),
    Double(f64),
    Object(HashMap<String,Value>),
    Array(Vec<Value>),
    Null
}
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match self {
            Value::String(val)=>{
                serializer.serialize_str(val.as_str())
            },
            Value::Long(val)=>{
                serializer.serialize_i64(*val)
            },
            Value::Double(val)=>{
                serializer.serialize_f64(*val)
            },
            Value::Null=>{
                serializer.serialize_none()
            },
            Value::Boolean(val)=>{
                serializer.serialize_bool(*val)
            },
            Value::Object(val)=>{
                serializer.serialize_some(val)
            },
            Value::Array(val)=>{
                serializer.serialize_some(val)
            }
        }
    }
}
#[derive(Debug,PartialEq,Clone,Serialize,Deserialize)]
pub struct RawVariableValue{
    pub value:Option<String>,
    pub name:String,
    pub data_type:VarType

}
impl RawVariableValue{
    pub fn is_valid(&self)-> bool {
        match &self.value {
            Option::None=> return true,
            _=>{}
        }
        match &self.data_type {
            VarType::String=>true,
            VarType::Long=>match self.value.clone().unwrap().parse::<i64>() {
                Ok(_)=>true,
                _=>false
            },
            VarType::Boolean=>match self.value.clone().unwrap().parse::<bool>() {
                Ok(_)=>true,
                _=>false
            },
            VarType::Double=>match self.value.clone().unwrap().parse::<f64>() {
                Ok(_)=>true,
                _=>false
            }
            _=>false
        }
    }
    pub fn to_value(&self)->Value{
        match &self.value {
            Option::None=> return Value::Null,
            _=>{}
        }
        match &self.data_type {
            VarType::String=>Value::String(self.value.clone().unwrap()),
            VarType::Long=>match self.value.clone().unwrap().parse::<i64>() {
                Ok(val)=>Value::Long(val),
                _=>Value::Null
            },
            VarType::Boolean=>match self.value.clone().unwrap().parse::<bool>() {
                Ok(val)=>Value::Boolean(val),
                _=>Value::Null
            },
            VarType::Double=>match self.value.clone().unwrap().parse::<f64>() {
                Ok(val)=>Value::Double(val),
                _=>Value::Null
            }
            _=>Value::Null
        }
    }
}

pub trait ValueProvider{
    fn read(&mut self,variable:Variable)->Value;

    //    fn iterate<F>(&mut self,refering_as:Variable,to_list:Variable,inner:F) where F:Fn();
    fn write(&mut self,text:String);
    fn set_index_ref(&mut self,index_ref_var:Variable,list_ref_var:Variable);
    fn drop(&mut self,str:String);
    fn load_ith_as(&mut self,i:usize,index_ref_var:Variable,list_ref_var:Variable);
    fn close(&mut self);
}
pub struct Environment<T> where T:ValueProvider {
    pub channel:Rc<RefCell<T>>
}
impl<T> Environment<T> where T:ValueProvider{
    pub fn new_rc(provider:T)->Environment<RCValueProvider<T>>{
        Environment{
            channel:Rc::new(RefCell::new(RCValueProvider{
                indexes:HashMap::new(),
                reference_store:Rc::new(RefCell::new(HashMap::new())),
                value_store:vec![],
                fallback_provider:provider
            }))
        }
    }
}
impl<T> Environment<T> where T:ValueProvider{
    pub fn iterate<F>(&self, refering_as: Variable, to_list: Variable, inner: F) where F: Fn(usize) {
        let length = (*self.channel).borrow_mut().read(Variable{
            name:format!("{}.size",to_list.name),
            data_type:Option::Some(VarType::Long)
        });
        (*self.channel).borrow_mut().set_index_ref(refering_as.clone() ,to_list.clone());
        match length {
            Value::Long(l)=>{
                let size = l as usize;

                for i in 0..size {
                    (*self.channel).borrow_mut().load_ith_as(i,refering_as.clone() ,to_list.clone());
                    inner(i);
                    (*self.channel).borrow_mut().drop(refering_as.name.clone());
                }
            },
            _=>{}
        }

    }
    pub fn build_iterate<F,G>(&self, refering_as: Variable, to_list: Variable,mut push_to:Vec<G>, inner: F)->Vec<G> where F: Fn(usize)->G{
        let length = (*self.channel).borrow_mut().read(Variable{
            name:format!("{}.size",to_list.name),
            data_type:Option::Some(VarType::Long)
        });
        (*self.channel).borrow_mut().set_index_ref(refering_as.clone() ,to_list.clone());
        match length {
            Value::Long(l)=>{
                let size = l as usize;
                for i in 0..size {
                    (*self.channel).borrow_mut().load_ith_as(i,refering_as.clone() ,to_list.clone());
                    push_to.push(inner(i));
                    (*self.channel).borrow_mut().drop(refering_as.name.clone());
                }
                return push_to;
            },
            _=>{return push_to;}
        }

    }
}

impl<T> ValueProvider for Environment<T> where T:ValueProvider{
    
    fn read(&mut self, variable: Variable) -> Value {
        (*self.channel).borrow_mut().read(variable)
    }

    fn write(&mut self, text: String) {
        (*self.channel).borrow_mut().write(text);
    }

    fn close(&mut self) {
        (*self.channel).borrow_mut().close();
    }
    fn drop(&mut self, str_val: String) { 
        (*self.channel).borrow_mut().drop(str_val);
    }
    fn set_index_ref(&mut self, as_var: Variable, in_var: Variable) { 
        (*self.channel).borrow_mut().set_index_ref(as_var,in_var);
    }

    fn load_ith_as(&mut self, i: usize, index_ref_var: Variable, list_ref_var: Variable) {
        (*self.channel).borrow_mut().load_ith_as(i,index_ref_var,list_ref_var);
    }
}

#[derive(Clone,PartialEq,Debug)]
pub struct Variable{
    pub name:String,
    pub data_type:Option<VarType>
}
#[derive(Debug,PartialEq,Clone)]
pub enum Object{
    Final(Value),
    Object(Rc<RefCell<HashMap<String,Rc<Object>>>>),
    List(Rc<RefCell<Vec<Rc<Object>>>>)
}
impl Object {
    pub fn new_list_object()->Object{
        return Object::List(Rc::new(RefCell::new(vec![])));
    }
    pub fn new_object_object(map:HashMap<String,Rc<Object>>)->Object{
        return Object::Object(Rc::new(RefCell::new(map)))
    }
}
#[derive(PartialEq,Clone)]
pub struct RCValueProvider<T> where T:ValueProvider{
    pub fallback_provider:T,
    pub value_store:Vec<Rc<Object>>,
    pub reference_store:Rc<RefCell<HashMap<String,Rc<Object>>>>,
    pub indexes:HashMap<String,String>,
}

impl<T> RCValueProvider<T> where T:ValueProvider{
    pub fn get_object_at_path(&self,var:String)->Option<Rc<Object>>{
        if var.contains('.'){
            let (left,right)=break_on(var.clone(),'.').unwrap();
            let opt_rc=self.get_object_at_path(left);
            match  opt_rc {
                Option::Some(rc)=>{
                    match &*rc {
                        Object::Object(obj)=>{
                            let ref_hm=(**obj).borrow_mut();
                            let ref_rc=ref_hm.get(&right);
                            match ref_rc {
                                Option::Some(a_rc)=> Option::Some(a_rc.clone()),
                                Option::None=>Option::None
                            }
                        },
                        Object::List(lst)=>{
                            if right==format!("size"){
                                Option::Some(Rc::new(Object::Final(Value::Long((**lst).borrow_mut().len() as i64))))
                            } else {
                                Option::None
                            }
                        },
                        _=>{Option::None}
                    }
                },
                Option::None=>{
                    Option::None
                }
            }

        } else {
            let temp=(*self.reference_store).borrow();
            let ref_rc=temp.get(&var);
            match ref_rc {
                Option::Some(a_rc)=> Option::Some(a_rc.clone()),
                Option::None=>Option::None
            }

        }

    }
    pub fn create_object_at_path(&self,var:String,object:Rc<Object>){
        if var.contains('.'){
            let (left,right)=break_on(var.clone(),'.').unwrap();
            let rc=self.get_object_at_path(left.clone());
            match rc {
                Option::Some(rc_object)=>{
                    match &*rc_object {
                        Object::Object(obj)=>{
                            (**obj).borrow_mut().insert(right,object);
                        },
                        _=>{
                            unimplemented!()
                        }
                    }
                },
                Option::None=>{
                    let mut map=HashMap::new();
                    map.insert(right,object);
                    let rc_obj=Rc::new(Object::new_object_object(map));
                    self.create_object_at_path(left,rc_obj)
                }
            }

        } else {
            (*self.reference_store).borrow_mut().insert(var.clone(),Rc::clone(&object));
            println!("Should Inserting in refered object {}, {:?}",var.clone(),self.indexes);
            if self.indexes.contains_key(&var.clone()){
                println!("Inserting in refered object {:?}",self.indexes);
                let obj=self.get_object_at_path(self.indexes.get(&var.clone()).unwrap().clone()).unwrap().clone();
                match &*obj {
                    Object::Final(val)=>{
                        unimplemented!()
                    },
                    Object::List(lst)=>{
                        (**lst).borrow_mut().push(Rc::clone(&object));
                    },
                    Object::Object(_)=>{
                        unimplemented!()
                    }

                }
            }
        }

    }



}
impl<T> ValueProvider for RCValueProvider<T> where T:ValueProvider{
    
    fn read(&mut self, var: Variable) -> Value {
        let obj = self.get_object_at_path(var.name.clone());
        match obj {
            Option::Some(rc_value)=>{
                match &*rc_value {
                    Object::Final(val)=>{
                        println!("Returning already found value{:?}",val);
                        val.clone()
                    },
                    _=>{
                        unimplemented!();
                    }
                }
            },
            Option::None =>{
                let opt = break_on(var.name.clone(),'.');
                match opt {
                    Option::Some((left,right))=>{
                        if right.clone() == "size"{
                            let obj= self.create_object_at_path(left,Rc::new(Object::new_list_object()));
                            println!("Reading from console");
                            let val=self.fallback_provider.read(var.clone());
                            val
                        } else {
                            println!("Reading from console");
                            let val=self.fallback_provider.read(var.clone());
                            self.create_object_at_path(var.name.clone(),Rc::new(Object::Final(val.clone())));
                            val
                        }
                    },
                    Option::None=>{
                        println!("Reading from console");
                        let val=self.fallback_provider.read(var.clone());
                        self.create_object_at_path(var.name.clone(),Rc::new(Object::Final(val.clone())));
                        val
                    }
                }

            }
        }

    }
    
    fn write(&mut self, str: String) {
         self.fallback_provider.write(str)
    }
    
    fn close(&mut self) { 
        self.fallback_provider.close();
    }
    fn set_index_ref(&mut self, as_ref:Variable, in_ref:Variable) { 
        self.indexes.insert(as_ref.name.clone(), in_ref.name.clone());
    }
    fn drop(&mut self, key: String) { 
        let mut keys_to_remove=Vec::new();
        for ref_key in (*self.reference_store).borrow().keys()  {
            if ref_key.starts_with(format!("{}.",key).as_str()){
                keys_to_remove.push(ref_key.clone())
            }
        }
        for rm_key in keys_to_remove{
            let mut temp=(*self.reference_store).borrow_mut();
            temp.remove(&rm_key);
        }
        let mut temp=(*self.reference_store).borrow_mut();
        temp.remove(&key);
    }

    fn load_ith_as(&mut self, i: usize, index_ref_var: Variable, list_ref_var: Variable) {
        println!("{:?}{:?}{:?}",i,index_ref_var,list_ref_var);
        if (*self.reference_store).borrow().contains_key(&list_ref_var.name.clone()){
            let val = (*self.reference_store).borrow().get(&list_ref_var.name.clone()).unwrap().clone();
            match &*val {
                Object::List(lst)=>{
                    let mut temp=(*self.reference_store).borrow_mut();
                    if (**lst).borrow().len()>i{
                        temp.insert(index_ref_var.name.clone(),(**lst).borrow().get(i).unwrap().clone());
                    } else if index_ref_var.data_type.clone().unwrap() == VarType::Object{
                        let map=HashMap::new();
                        let obj = Rc::new(Object::new_object_object(map));
                        (**lst).borrow_mut().push(obj.clone());
                        temp.insert(index_ref_var.name.clone(),obj);
                    } else if index_ref_var.data_type.clone().unwrap() == VarType::List {
                        let objct = Rc::new(Object::new_list_object());
                        (**lst).borrow_mut().push(objct.clone());
                        temp.insert(index_ref_var.name.clone(),objct);
                    }
                },
                _=>{unimplemented!()}
            }
        } else {
            let lst=Rc::new(Object::new_list_object());
            self.create_object_at_path(list_ref_var.name.clone(),lst.clone());
            match &*lst {
                Object::List(lst)=>{
                    let mut temp=(*self.reference_store).borrow_mut();
                    if (**lst).borrow().len()>i{
                        temp.insert(index_ref_var.name.clone(),(**lst).borrow().get(i).unwrap().clone());
                    } else if index_ref_var.data_type.clone().unwrap() == VarType::Object{
                        let map=HashMap::new();
                        let obj = Rc::new(Object::new_object_object(map));
                        (**lst).borrow_mut().push(obj.clone());
                        temp.insert(index_ref_var.name.clone(),obj);
                    } else if index_ref_var.data_type.clone().unwrap() == VarType::List {
                        let objct = Rc::new(Object::new_list_object());
                        (**lst).borrow_mut().push(objct.clone());
                        temp.insert(index_ref_var.name.clone(),objct);
                    }
                },
                _=>{unimplemented!()}
            }
        }

    }
}

#[cfg(test)]
mod tests{
    use crate::runtime::RCValueProvider;
    use crate::runtime::Variable;
    use crate::runtime::Environment;
    use crate::runtime::{ValueProvider, VarType};
    use std::collections::HashMap;
    use crate::runtime::Value;
    use std::rc::Rc;
    use std::cell::RefCell;

    extern crate serde_json;
    #[test]
    fn should_serialize_string_value(){
        assert_eq!(serde_json::to_string(&Value::String(format!("hello"))).unwrap(),format!("\"hello\""))
    }
    #[test]
    fn should_serialize_long_value(){
        assert_eq!(serde_json::to_string(&Value::Long(34)).unwrap(),format!("34"))
    }
    #[test]
    fn should_serialize_double_value(){
        assert_eq!(serde_json::to_string(&Value::Double(34.00)).unwrap(),format!("34.0"))
    }
    #[test]
    fn should_serialize_null_value(){
        assert_eq!(serde_json::to_string(&Value::Null).unwrap(),format!("null"))
    }
    #[test]
    fn should_serialize_object_value(){
        let mut map=HashMap::new();
        map.insert(format!("hello"),Value::String(format!("hello")));
        assert_eq!(serde_json::to_string(&Value::Object(map)).unwrap(),format!(r#"{{"hello":"hello"}}"#))
    }
    #[test]
    fn should_serialize_array_value(){
        let mut array=Vec::new();
        array.push(Value::String(format!("hello")));
        assert_eq!(serde_json::to_string(&Value::Array(array)).unwrap(),format!(r#"["hello"]"#))
    }

    impl ValueProvider for MockProvider{
        
        fn read(&mut self, _: Variable) -> Value { let ret=self.1[self.0].clone(); self.0+=1;ret }
        fn write(&mut self, str: String) { println!("{}",str) }
        fn close(&mut self) {  }
        fn set_index_ref(&mut self, _: Variable, _:Variable) { }
        fn drop(&mut self, _: String) { }

        fn load_ith_as(&mut self, i: usize, index_ref_var: Variable, list_ref_var: Variable) {

        }
    }
    #[derive(Debug)]
    struct MockProvider(usize,Vec<Value>);
    #[test]
    fn should_iterate_over_runtime(){
        let rt=Environment{
            channel: Rc::new(RefCell::new(MockProvider(0,vec![Value::Long(3),
            Value::String(format!("Atmaram 0")),
            Value::Long(2),
            Value::String(format!("Atmaram 00")),
            Value::String(format!("Atmaram 01")),
            Value::String(format!("Atmaram 1")),
            Value::Long(2),
            Value::String(format!("Atmaram 10")),
            Value::String(format!("Atmaram 11")),
            Value::String(format!("Atmaram 2")),
            Value::Long(2),
            Value::String(format!("Atmaram 20")),
            Value::String(format!("Atmaram 21"))
            ])))
        };
        let mch = Rc::clone(&rt.channel);
        rt.iterate(Variable{
            name:format!("hobby"),
            data_type:Option::Some(VarType::String)
        }, Variable{
            name:format!("hobbies"),
            data_type:Option::Some(VarType::List)
        }, |i| {
            assert_eq!(mch.borrow_mut().read(Variable{
                name:format!("name"),
                data_type:Option::Some(VarType::String)
            }),Value::String(format!("Atmaram {}",i)));
            rt.iterate(Variable{
                name:format!("category"),
                data_type:Option::Some(VarType::String)
            }, Variable{
                name:format!("categories"),
                data_type:Option::Some(VarType::List)
            }, |j| {
                assert_eq!(mch.borrow_mut().read(Variable{
                    name:format!("name"),
                    data_type:Option::Some(VarType::String)
                }),Value::String(format!("Atmaram {}{}",i,j)))
            });
        })
    }

    #[test]
    fn should_read_same_variable_from_rc_provider_multiple_times(){
        let mut rcp=RCValueProvider {
            value_store:Vec::new(),
            reference_store:Rc::new(RefCell::new(HashMap::new())),
            fallback_provider:MockProvider(0,vec![
            Value::String(format!("Atmaram"))
            ]),
            indexes:HashMap::new()
        };
        let var =  Variable {
            name:format!("name"),
            data_type:Option::Some(VarType::String)
        };
        assert_eq!(rcp.read(var.clone()),Value::String(format!("Atmaram")));
        assert_eq!(rcp.read(var.clone()),Value::String(format!("Atmaram")));
    }
    #[test]
    fn should_size_from_rc_provider(){
        let mut rcp=RCValueProvider {
            value_store:Vec::new(),
            reference_store:Rc::new(RefCell::new(HashMap::new())),
            fallback_provider:MockProvider(0,vec![
                Value::Long(2),
                Value::String(format!("Atmaram"))
            ]),
            indexes:HashMap::new()
        };
        let var =  Variable {
            name:format!("name.size"),
            data_type:Option::Some(VarType::String)
        };
        assert_eq!(rcp.read(var.clone()),Value::Long(2));
    }

    #[test]
    fn should_iterate_over_runtime_reading_values(){
        let mut rt=Environment{
            channel: Rc::new(RefCell::new(
                RCValueProvider {
                    value_store:Vec::new(),
                    reference_store:Rc::new(RefCell::new(HashMap::new())),
                    fallback_provider:MockProvider(0,vec![
                        Value::Long(2),
                        Value::String(format!("Atmaram 0")),
                        Value::String(format!("Atmaram 1"))
                    ]),
                    indexes:HashMap::new()
                }
            ))
        };
        let mch = Rc::clone(&rt.channel);
        rt.iterate(Variable{
            name:format!("hobby"),
            data_type:Option::Some(VarType::String)
        }, Variable{
            name:format!("hobbies"),
            data_type:Option::None
        }, |i| {
            assert_eq!(mch.borrow_mut().read(Variable{
                name:format!("hobby.name"),
                data_type:Option::Some(VarType::String)
            }),Value::String(format!("Atmaram {}",i)));
        });
        rt.iterate(Variable{
            name:format!("hobby"),
            data_type:Option::Some(VarType::String)
        }, Variable{
            name:format!("hobbies"),
            data_type:Option::None
        }, |i| {
            println!("helo");
            assert_eq!(mch.borrow_mut().read(Variable{
                name:format!("hobby.name"),
                data_type:Option::Some(VarType::String)
            }),Value::String(format!("Atmaram {}",i)));
        });

    }
}