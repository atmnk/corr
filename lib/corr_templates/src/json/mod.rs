pub mod parser;
pub mod extractable;
extern crate corr_core;
use corr_core::runtime::Variable;
use corr_core::runtime::Value;
use corr_core::runtime::ValueProvider;
use corr_core::runtime::Environment;
use std::collections::HashMap;
use std::fmt::Debug;
use crate::{get_function};

#[derive(Clone,PartialEq,Debug)]
pub enum Producer{
    Json(Json),
    JsonArrayProducer(JsonArrayProducer),
    JsonArrayTimesProducer(JsonTimesProducer)
}
#[derive(Clone,PartialEq,Debug)]
pub struct JsonArrayProducer{
    as_var:Variable,
    in_var:Variable,
    inner_producer:Box<Producer>
}
#[derive(Clone,PartialEq,Debug)]
pub struct JsonTimesProducer{
    as_var:Variable,
    in_var:Variable,
    times:usize,
    counter_var:Variable,
    inner_producer:Box<Producer>
}
#[derive(Clone,PartialEq,Debug)]
pub struct Function{
    name:String,
    args:Vec<Argument>
}

#[derive(Clone,PartialEq,Debug)]
pub enum Argument{
    Variable(Variable),
    Function(Function),
    Final(Value)
}
#[derive(Clone,PartialEq,Debug)]
pub enum Json {
    Constant(Value),
    Variable(Variable),
    Function(Function),
    TemplatedStaticArray(Vec<Json>),
    TemplatedDynamicArray(JsonArrayProducer),
    TemplatedTimesDynamicArray(JsonTimesProducer),
    Object(HashMap<String,Json>)
}
pub trait Fillable<T> {
    fn fill(&self,runtime:&Environment)->T;
}
impl Fillable<Value> for Value {
    fn fill(&self, _runtime: &Environment) -> Value {
        self.clone()
    }
}
impl Fillable<Value> for Producer{
    fn fill(&self, runtime:&Environment) ->Value {
        match self {
            Producer::Json(json)=>{
                Value::Array(vec![json.fill(runtime)])
            },
            Producer::JsonArrayProducer(jap)=>{
                jap.fill(runtime)
            },
            Producer::JsonArrayTimesProducer(jtp)=>{
                jtp.fill(runtime)
            }
        }
    }
}
impl Fillable<Value> for JsonArrayProducer{
    fn fill(&self, runtime:&Environment) ->Value {
        let res=Vec::new();
        let res=runtime.build_iterate(self.as_var.clone(), self.in_var.clone(),res, |_|{
            self.inner_producer.fill(&runtime)
        });
        Value::Array(res.into_iter().map(|v|{
            match v {
                Value::Array(l)=>l,
                _=>vec![]
            }
        }).flatten().collect())
    }
}
impl Fillable<Value> for JsonTimesProducer{
    fn fill(&self, runtime:&Environment) ->Value {
        let res=runtime.build_iterate_outside_building_inside(self.as_var.clone(), self.in_var.clone(),self.times.clone(), |i|{
            let val=Value::Long(i as i64);
            (*runtime.channel).borrow_mut().load_value_as(self.counter_var.clone() ,val);
            self.inner_producer.fill(&runtime)
        });
        Value::Array(res.into_iter().map(|v|{
            match v {
                Value::Array(l)=>l,
                _=>vec![]
            }
        }).flatten().collect())
    }
}
impl Fillable<Value> for Function {
    fn fill(&self, runtime:&Environment) ->Value {
        let mut args=Vec::new();
        for arg in &self.args{
            args.push(arg.fill(runtime))
        }
        get_function(self.name.clone()).eval(args)
    }
}
impl Fillable<Value> for Argument {
    fn fill(&self, runtime:&Environment) ->Value {
        match self {
            Argument::Function(fun)=>{
                fun.fill(runtime)
            },
            Argument::Variable(var)=>{
                runtime.channel.borrow_mut().read(var.clone())
            },
            Argument::Final(val)=>{
                val.clone()
            }
        }
    }
}
impl Fillable<Value> for Json {
    fn fill(&self, runtime:&Environment) ->Value {
        match self {
            Json::Constant(val)=>{
                val.fill(runtime)
            },
            Json::Variable(var)=>{
                runtime.channel.borrow_mut().read(var.clone())
            },
            Json::TemplatedStaticArray(inner_objects)=>{
                let mut final_res=Vec::new();
                for object in inner_objects {
                    let filled_obj=object.fill(runtime);
                    final_res.push(filled_obj);
                }
                Value::Array(final_res)
            },
            Json::TemplatedDynamicArray(jap)=>{
                jap.fill(runtime)
            },
            Json::TemplatedTimesDynamicArray(jtp)=>{
                jtp.fill(runtime)
            },
            Json::Object(map)=>{
                let mut res=HashMap::new();
                for (key,value) in map{
                    res.insert(key.clone(), value.fill(runtime));
                }
                Value::Object(res)
            },
            Json::Function(fun)=>{
                fun.fill(runtime)
            }
        }
    }
}
#[cfg(test)]
mod tests{

    use corr_core::runtime::{Environment, Value,ValueProvider,Variable, VarType};
    use crate::json::{Json, Fillable, JsonArrayProducer, Producer};
    use std::collections::HashMap;


    impl ValueProvider for MockProvider{
        
        fn read(&mut self, var: Variable) -> Value { 
            for (val,value) in &self.0 {
                if *val == var.name {
                    return value.clone()
                }
            }
            return Value::Null
        }
        fn write(&mut self, str: String) { println!("{}",str) }
        fn close(&mut self) { unimplemented!() }
        fn set_index_ref(&mut self, _: Variable, _: Variable) { unimplemented!() }
        fn drop(&mut self, _: std::string::String) { unimplemented!() }

        fn load_ith_as(&mut self, _i: usize, _index_ref_var: Variable, _list_ref_var: Variable) {
            unimplemented!()
        }

        fn save(&self, _var: Variable, _value: Value) {
            unimplemented!()
        }

        fn load_value_as(&mut self, _ref_var: Variable, _val: Value) {
            unimplemented!()
        }
    }
    #[derive(Debug)]
    struct MockProvider(Vec<(String,Value)>);


    #[test]
    fn serde_test(){
        let jsondoc = r#"
        {
            "age":22.0
        }
    "#;

        // Parse JSON document
        let json: serde_json::Value = serde_json::from_str(jsondoc).unwrap();
        println!("{},{:?}",serde_json::to_string(&json).unwrap(),json);
    }
    #[test]
    fn should_fill_dynamic_array(){
        let rt=Environment::new_rc(MockProvider(vec![(format!("hobbies.size"),Value::Long(3)),
                                                     (format!("hobby"),Value::String(format!("Atmaram")))]));
        let tmp=Json::TemplatedDynamicArray(
            JsonArrayProducer{
                as_var:Variable {
                    name:format!("hobby"),
                    data_type: Option::Some(VarType::String)
                },
                in_var: Variable {
                    name:format!("hobbies"),
                    data_type: Option::None
                },
                inner_producer:Box::new(Producer::Json(Json::Variable(Variable {
                    name:format!("hobby"),
                    data_type: Option::Some(VarType::String)
                })))
            });
        let mut vc=Vec::new();
        vc.push(Value::String(format!("Atmaram")));
        vc.push(Value::String(format!("Atmaram")));
        vc.push(Value::String(format!("Atmaram")));
        assert_eq!(tmp.fill(&rt),Value::Array(vc))
    }

    #[test]
    fn should_fill_static_array(){
        let rt=Environment::new_rc(MockProvider(vec![(format!("hobbies.size"),Value::Long(3)),
                                                     (format!("name 0"),Value::String(format!("Atmaram 00"))),
                                                     (format!("name 1"),Value::String(format!("Atmaram 01"))),
                                                     (format!("name 2"),Value::String(format!("Atmaram 02"))),
        ]));
        let mut vec=Vec::new();
        vec.push(Json::Variable(Variable { 
            name:format!("name 0"),
            data_type: Option::Some(VarType::String)
        }));
        vec.push(Json::Variable(Variable { 
            name:format!("name 1"),
            data_type: Option::Some(VarType::String)
        }));
        vec.push(Json::Variable(Variable { 
            name:format!("name 2"),
            data_type: Option::Some(VarType::String)
        }));
        let tmp=Json::TemplatedStaticArray(vec);
        let mut vc=Vec::new();
        vc.push(Value::String(format!("Atmaram 00")));
        vc.push(Value::String(format!("Atmaram 01")));
        vc.push(Value::String(format!("Atmaram 02")));
        assert_eq!(tmp.fill(&rt),Value::Array(vc))
    }
    #[test]
    fn should_fill_object(){
        let rt=Environment::new_rc(MockProvider(vec![(format!("hobbies.size"),Value::Long(3)),
                                                     (format!("fn"),Value::String(format!("Atmaram"))),
                                                     (format!("ln"),Value::String(format!("Naik"))),
        ]));
        let mut map=HashMap::new();
        map.insert(format!("fn"), Json::Variable(Variable { 
            name:format!("fn"),
            data_type: Option::Some(VarType::String)
        }));
        map.insert(format!("ln"), Json::Variable(Variable { 
            name:format!("ln"),
            data_type: Option::Some(VarType::String)
        }));
        
        let tmp=Json::Object(map);
        let mut valmap=HashMap::new();
        valmap.insert(format!("fn"), Value::String(format!("Atmaram")));
        valmap.insert(format!("ln"), Value::String(format!("Naik")));
        let val=tmp.fill(&rt);
        assert_eq!(val,Value::Object(valmap));
        println!("{:?}",val);
    }

}

