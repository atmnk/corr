


use crate::core::runtime::Context;
use crate::core::{Variable, convert, DataType, Value};

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
                let mut remaining = input;
                let mut vars_to_define = vec![] ;
                if let Some(f) = first {
                    if let Some((i,_t))=dynamic_tag (remaining,f.clone()).ok(){
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

                    } else {
                        return false;
                    }
                }
                if let Some(v) = last {
                    if let Some(val) = convert(v.name.clone(),remaining.to_string(),v.data_type.unwrap_or(DataType::String)) {
                        if val.value.eq(&Value::Null) || val.value.eq(&Value::String("".to_string())){
                            return false
                        } else {
                            vars_to_define.push(val);
                        }
                    } else {
                        return false;
                    }
                } else if remaining.len() > 0 {
                    return false;
                }
                for val in vars_to_define{
                    context.define(val.name,val.value).await;
                }
                true
            }
        }

    }
}
#[cfg(test)]
mod tests{
    use std::sync::{Arc, Mutex};
    
    use crate::parser::Parsable;

    use crate::core::{Value};
    use crate::core::proto::Output;
    use crate::core::runtime::Context;
    use crate::template::text::extractable::ExtractableText;

    #[tokio::test]
    async fn should_extract_extractable_text_with_one_variable(){
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let text=r#"text `<%name%>`"#;
        let (_i,a)=ExtractableText::parser(text).unwrap();
        assert_eq!(a.capture("Atmaram",&context).await,true);
        assert_eq!(context.get_var_from_store(format!("name")).await,Option::Some(Value::String(format!("Atmaram"))));
    }
    #[tokio::test]
    async fn should_extract_extractable_text_with_single_variable_and_leading_string(){
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let text=r#"text `http://localhost:8090/<%name%>`"#;
        let (_i,a)=ExtractableText::parser(text).unwrap();
        assert_eq!(a.capture("http://localhost:8090/Atmaram",&context).await,true);
        assert_eq!(context.get_var_from_store(format!("name")).await,Option::Some(Value::String(format!("Atmaram"))));
    }
    #[tokio::test]
    async fn should_extract_extractable_text_with_single_variable_and_trailing_string(){
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let text=r#"text `<%code%>/salary`"#;
        let (_i,a)=ExtractableText::parser(text).unwrap();
        assert_eq!(a.capture("E001/salary",&context).await,true);
        assert_eq!(context.get_var_from_store(format!("code")).await,Option::Some(Value::String(format!("E001"))));
    }
    #[tokio::test]
    async fn should_extract_extractable_text_with_single_variable_and_leading_and_trailing_string(){
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let text=r#"text `http://localhost:8090/employee/<%code%>/salary`"#;
        let (_i,a)=ExtractableText::parser(text).unwrap();
        assert_eq!(a.capture("http://localhost:8090/employee/E001/salary",&context).await,true);
        assert_eq!(context.get_var_from_store(format!("code")).await,Option::Some(Value::String(format!("E001"))));
    }
    #[tokio::test]
    async fn should_not_match_part_of_static_text(){
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let text=r#"text `http://localhost:8090/employee`"#;
        let (_i,a)=ExtractableText::parser(text).unwrap();
        assert_eq!(a.capture("http://localhost:8090/employee/E001/salary",&context).await,false);
    }
    #[tokio::test]
    async fn should_match_part_of_text_with_variable_trailing_with_nothing(){
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let text=r#"text `http://localhost:8090/employee/<%code%>`"#;
        let (_i,a)=ExtractableText::parser(text).unwrap();
        assert_eq!(a.capture("http://localhost:8090/employee/E001/salary",&context).await,true);
    }
    #[tokio::test]
    async fn should_not_match_part_of_text_if_trailing_text_not_present(){
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let text=r#"text `http://localhost:8090/employee/<%code%>/salary`"#;
        let (_i,a)=ExtractableText::parser(text).unwrap();
        assert_eq!(a.capture("http://localhost:8090/employee/",&context).await,false);
    }
    #[tokio::test]
    async fn should_not_match_part_of_text_with_variable_trailing_with_string(){
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let text=r#"text `http://localhost:8090/employee/<%code%>/sal`"#;
        let (_i,a)=ExtractableText::parser(text).unwrap();
        assert_eq!(a.capture("http://localhost:8090/employee/E001/salary",&context).await,false);
    }
    #[tokio::test]
    async fn should_not_match_if_trailing_variable_not_present(){
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let text=r#"text `http://localhost:8090/employee/<%code%>`"#;
        let (_i,a)=ExtractableText::parser(text).unwrap();
        assert_eq!(a.capture("http://localhost:8090/employee/",&context).await,false);
    }
    #[tokio::test]
    async fn should_match_if_middle_variable_not_present(){
        let input=vec![];
        let buffer:Arc<Mutex<Vec<Output>>> = Arc::new(Mutex::new(vec![]));
        let context=Context::mock(input,buffer.clone());
        let text=r#"text `http://localhost:8090/employee/<%code%>/`"#;
        let (_i,a)=ExtractableText::parser(text).unwrap();
        assert_eq!(a.capture("http://localhost:8090/employee//",&context).await,true);
    }
    // #[test]
    // fn should_parse_extractable_text_with_multiple_variable(){
    //     let text=r#"text `Hello<%name%>World`"#;
    //     let a=ExtractableText::parser(text);
    //     assert_if(text,a,ExtractableText::Multi(Option::Some("Hello".to_string()),vec![(Variable::new("name"),"World".to_string())],Option::None))
    // }
}
