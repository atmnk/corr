use std::str;
use nom::character::complete::{char, multispace0, digit1, anychar};
use nom::{IResult,  InputTakeAtPosition, AsChar};
use nom::error::ParseError;
use nom::bytes::complete::{tag, is_not, escaped_transform, escaped};
use nom::branch::alt;
use nom::sequence::{tuple, terminated, delimited, separated_pair,preceded};
use nom::combinator::{map, opt};
use corr_core::runtime::{ Variable, VarType};
use nom::multi::many0;
use corr_rest::{PostStep, GetStep, RestData, PutStep, PatchStep, DeleteStep, BodyData};
use corr_journeys::{Executable, Journey, LoopStep, PrintStep, TimesStep};
use std::fs::File;
use std::io::Read;
use nom::lib::std::collections::HashMap;
use corr_templates::text::Text;
use corr_templates::json::Json;
use corr_templates::json::extractable::ExtractableJson;

type ParserError<'a, T> = Result<(&'a [u8], T), nom::Err<(&'a [u8], nom::error::ErrorKind)>>;
fn non_ascii(chr: u8) -> bool {
    chr >= 0x80 && chr <= 0xFD
}
pub fn ws<I, O, E: ParseError<I>, F>(inner: F) -> impl Fn(I) -> IResult<I, O, E>
    where
        F: Fn(I) -> IResult<I, O, E>,
        I: InputTakeAtPosition,
        <I as InputTakeAtPosition>::Item: AsChar + Clone,
{
    move |input: I| {
        let (input, _) = multispace0(input)?;
        terminated(&inner,multispace0)(input)
    }
}

fn identifier(input: &[u8]) -> ParserError<String> {
    if !nom::character::is_alphabetic(input[0]) && input[0] != b'_' && input[0] != b'.' && !non_ascii(input[0]) {
        return Err(nom::Err::Error(error_position!(
            input,
            nom::error::ErrorKind::AlphaNumeric
        )));
    }
    for (i, ch) in input.iter().enumerate() {
        if i == 0 || nom::character::is_alphanumeric(*ch) || *ch == b'_'|| *ch == b'.' || non_ascii(*ch) {
            continue;
        }
        return Ok((&input[i..], str::from_utf8(&input[..i]).unwrap().to_string()));
    }
    Ok((&input[1..], str::from_utf8(&input[..1]).unwrap().to_string()))
}
fn name(input: &[u8]) -> IResult<&[u8],String> {
    alt((name_lit,
         identifier
    ))(input)
}
fn name_lit(i: &[u8]) -> IResult<&[u8], String> {
    ws(delimited(char('`'),inner_name,char('`')))(i)
}
fn inner_name(i:&[u8])->IResult<&[u8],String>{
    map(escaped_transform(is_not("\\`"), '\\', |i: &[u8]| alt((tag("`"),tag("\\")))(i)),
        |abc| str::from_utf8(&abc).unwrap().to_string())(i)
}
fn block_steps(i:&[u8])->IResult<&[u8],Vec<Box<dyn Executable>>>{
    many0(
        terminated(ws(step),ws(char(';')))
    )(i)
}
fn block(i:&[u8])->IResult<&[u8],Vec<Box<dyn Executable>>>{
    delimited(ws(tag("{")),ws(block_steps),ws(tag("}")))(i)
}
fn journey(i:&[u8])->IResult<&[u8],Journey> {
    let (i,(j_name,j_steps))=tuple((ws(name),ws(block)))(i)?;
    Ok((i,Journey{
        name:j_name,
        steps:j_steps
    }))
}
#[derive(Debug)]
pub enum Argument{
    Text(Text),
    Json(Json),
    ExtractableJson(ExtractableJson),
    Nil,
    Map(HashMap<String,Argument>)
}
fn var_type(i: &[u8]) -> IResult<&[u8], Option<VarType>> {
    map(ws(alt((tag("List"),tag("Object"),tag("Long"), tag("Double"),tag("Boolean"),tag("String")))), |s| {
        let val = str::from_utf8(s);
        match  val {
            Ok(inner_tag) => match inner_tag {
                "Long" => Option::Some(VarType::Long),
                "Double" => Option::Some(VarType::Double),
                "Boolean" => Option::Some(VarType::Boolean),
                "String" => Option::Some(VarType::String),
                "List" => Option::Some(VarType::List),
                "Object" => Option::Some(VarType::Object),
                _=> Option::None
            }
            _ => Option::None
        }
    })(i)
}
fn times_call(i:&[u8])->IResult<&[u8],Box<dyn Executable>>{
    let fun = tuple((
        ws(tag("times")),
        ws(tag("(")),
        ws(long_lit),
        ws(tag(",")),
        ws(identifier),
        ws(tag(",")),
        ws(identifier),
        ws(tag(":")),
        ws(var_type),
        ws(tag("in")),
        ws(identifier),
        ws(tag(")")),
        block
    ));
    let (i,(_,_,times,_,couner,_,with,_,vt,_,on,_,block)) = fun(i)?;
    let tt=TimesStep{
        as_var:Variable{
            name:with.to_string(),
            data_type:vt
        },
        in_var:Variable{
            name:on.to_string(),
            data_type:Option::Some(VarType::List)
        },
        counter_var:Variable{
            name:couner.to_string(),
            data_type:Option::Some(VarType::Long)
        },
        times:times as usize,
        inner_steps:block
    };
    Ok((i,Box::new(tt)))
}
fn long_lit(i: &[u8]) -> IResult<&[u8], i64> {
    let num = ws(tuple((opt(tag("-")),digit1)));
    let (i,(sign,nums)) = num(i)?;
    match sign.map(|s| str::from_utf8(s).unwrap()).unwrap_or("") {
        "-"=>Ok((i,str::from_utf8(nums).unwrap().parse::<i64>().unwrap()*-1)),
        _=>Ok((i,str::from_utf8(nums).unwrap().parse::<i64>().unwrap()))
    }

}
fn loop_call(i:&[u8])->IResult<&[u8],Box<dyn Executable>>{
    let fun = tuple((
        ws(tag("for")),
        ws(tag("(")),
        ws(identifier),
        ws(tag(":")),
        ws(var_type),
        ws(tag("in")),
        ws(identifier),
        ws(tag(")")),
        block
    ));
    let (i,(_,_,with,_,vt,_,on,_,blocks)) = fun(i)?;
    let lst=LoopStep{
        as_var:Variable{
            name:with.to_string(),
            data_type:vt
        },
        in_var:Variable{
            name:on.to_string(),
            data_type:Option::Some(VarType::List)
        },
        inner_steps:blocks
    };
    Ok((i,Box::new(lst)))
}
fn step(i:&[u8])->IResult<&[u8],Box<dyn Executable>> {
    alt((
    loop_call,
    times_call,
    function_call))(i)
}
fn text_template_arg(i:&[u8])->IResult<&[u8],Argument> {
    map(preceded(tag("@text\""),terminated(corr_templates::text::parser::text,tag("\""))),|val|{
        Argument::Text(val)})(i)
}
fn json_template_arg(i:&[u8])->IResult<&[u8],Argument> {
    map(preceded(tag("@json"),ws(corr_templates::json::parser::json)),|val|Argument::Json(val))(i)
}
fn map_arg(i:&[u8])->IResult<&[u8],Argument> {
    let fun= tuple(
        (
            tuple((ws(tag("@map")),ws(tag("{")))),
            ws(tuple((
                many0(terminated(tuple((
                    ws(string_lit),
                    ws(tag(":")),
                    ws(arg)
                )),ws(tag(",")))),
                opt(tuple((
                    ws(string_lit),
                    ws(tag(":")),
                    ws(arg))))))),
            ws(tag("}"))));
    let (i,(_,(pairs,opt_last_pair),_))=fun(i)?;
    let mut map=HashMap::new();
    for (key,_,value)  in pairs  {
        map.insert(String::from(key),value);
    }
    if let Some((key,_,value))=opt_last_pair{
        map.insert(String::from(key),value);
    }

    return Ok((i,Argument::Map(map)));
}
fn ejson_template_arg(i:&[u8])->IResult<&[u8],Argument> {
    map(preceded(tag("@ejson"),ws(corr_templates::json::extractable::parser::json)),|val|Argument::ExtractableJson(val))(i)
}
fn nil_arg(i:&[u8])->IResult<&[u8],Argument> {
    let (i,_)=tag("@nil")(i)?;
    Ok((i,Argument::Nil))
}
fn arg(i:&[u8])->IResult<&[u8],Argument> {
    alt(
        (text_template_arg,
         text_template_arg,
        json_template_arg,
         ejson_template_arg,
         nil_arg,
         map_arg
        )
    )(i)
}
fn named_arg(i:&[u8])->IResult<&[u8],(String,Argument)> {
    separated_pair(identifier,tag(":"),arg)(i)
}
fn args(i:&[u8])->IResult<&[u8],HashMap<String,Argument>>{
    let fun = tuple((ws(tag("(")),many0(terminated(named_arg,ws(tag(",")))),opt(named_arg),ws(tag(")"))));
    let (i,(_,start_pairs,opt_last_pair,_)) = fun(i)?;
    let mut  map=HashMap::new();
    for  (key,value) in start_pairs {
        map.insert(key,value);
    }
    if opt_last_pair.is_some() {
        let (key,value)=opt_last_pair.unwrap();
        map.insert(key,value);
    }
    Ok((i,map))

}
fn function_call(i:&[u8])->IResult<&[u8],Box<dyn Executable>>{
    let (i,(f_name,f_args))=tuple((name,args))(i)?;
    Ok((i,resolve(f_name,f_args)))
}
fn get_restData(args:HashMap<String,Argument>)->RestData{
    let response=match args.get(&format!("response")) {
        Option::Some(arg_val)=>{
            match arg_val {
                Argument::ExtractableJson(val)=>{
                    Option::Some(val.clone())
                },
                _=>{Option::None}
            }
        }
        _=>unimplemented!()
    };
    let url=match args.get(&format!("url")) {
        Option::Some(arg_val)=>{
            match arg_val {
                Argument::Text(val)=>{
                    val.clone()
                },
                _=>{unimplemented!()}
            }
        }
        _=>unimplemented!()
    };
    let opt_header_args=match args.get(&format!("headers")) {
        Option::Some(arg_val)=>{
            match arg_val {
                Argument::Map(val)=>{
                    Option::Some(val.clone())
                },
                _=>{unimplemented!()}
            }
        }
        Option::None=> Option::None
    };
    let mut headers=HashMap::new();
    if let Some(header_args) = opt_header_args {
        for (key,value) in header_args {
            if let Argument::Text(text_val) = value{
                headers.insert(key.clone(),text_val.clone());
            }
        }
    }
    return RestData{
            url,
            response,
            headers
        }
}
fn resolve_post(args:HashMap<String,Argument>)->Box<dyn Executable>{
    let body=match args.get(&format!("body")) {
        Option::Some(arg_val)=>{
            match arg_val {
                Argument::Json(val)=>{
                    BodyData::Json(val.clone())
                },
                Argument::Text(val)=>{
                    BodyData::Text(val.clone())
                },
                _=>{
                    unimplemented!()
                }
            }
        }
        _=>unimplemented!()

    };
    Box::new(PostStep{
        rest:get_restData(args),
        body
    })
}
fn resolve_put(args:HashMap<String,Argument>)->Box<dyn Executable>{
    let body=match args.get(&format!("body")) {
        Option::Some(arg_val)=>{
            match arg_val {
                Argument::Json(val)=>{
                    BodyData::Json(val.clone())
                },
                Argument::Text(val)=>{
                    BodyData::Text(val.clone())
                },
                _=>{
                    unimplemented!()
                }
            }
        }
        _=>unimplemented!()

    };
    Box::new(PutStep{
        rest:get_restData(args),
        body
    })
}
fn resolve_patch(args:HashMap<String,Argument>)->Box<dyn Executable>{
    let body=match args.get(&format!("body")) {
        Option::Some(arg_val)=>{
            match arg_val {
                Argument::Json(val)=>{
                    BodyData::Json(val.clone())
                },
                Argument::Text(val)=>{
                    BodyData::Text(val.clone())
                },
                _=>{
                    unimplemented!()
                }
            }
        }
        _=>unimplemented!()

    };
    Box::new(PatchStep{
        rest:get_restData(args),
        body
    })
}
fn resolve_get(args:HashMap<String,Argument>)->Box<dyn Executable>{
    Box::new(GetStep{
        rest:get_restData(args)
    })
}
fn resolve_delete(args:HashMap<String,Argument>)->Box<dyn Executable>{
    Box::new(DeleteStep{
        rest:get_restData(args)
    })
}
fn string_lit(i: &[u8]) -> IResult<&[u8], &str> {
    map(
        ws(delimited(
            char('\"'),
            opt(escaped(is_not("\\\""), '\\', anychar)),
            char('\"'),
        )),
        |s| s.map(|s| str::from_utf8(s).unwrap()).unwrap_or(""),
    )(i)
}
fn resolve_print(args:HashMap<String,Argument>)->Box<dyn Executable>{
    let text=match args.get(&format!("text")) {
        Option::Some(arg_val)=>{
            match arg_val {
                Argument::Text(val)=>{
                    val.clone()
                },
                _=>{unimplemented!()}
            }
        }
        _=>unimplemented!()

    };
    Box::new(PrintStep{
        text
    })
}
fn resolve(name:String,args:HashMap<String,Argument>)->Box<dyn Executable>{
    match &name[..] {
        "post"=>resolve_post(args),
        "get"=>resolve_get(args),
        "put"=>resolve_put(args),
        "patch"=>resolve_patch(args),
        "delete"=>resolve_delete(args),
        "print"=>resolve_print(args),
        _=>unimplemented!()
    }
}
pub fn read_journey_from_file(mut file:File)->Journey{
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (_,val)=journey(contents.as_bytes()).unwrap();
    return val;
}
#[cfg(test)]
mod tests{
    use crate::parser::{name, journey, step, text_template_arg, json_template_arg, nil_arg, Argument, ejson_template_arg};
    use corr_core::runtime::{ValueProvider, Environment, Variable, Value};
    use corr_journeys::Executable;
    use nom::AsBytes;

    #[test]
    fn should_parse_identifier(){
        let (_,k)=name("`atmaram \\`naik`".as_bytes()).unwrap();
        assert_eq!(k,"atmaram `naik")
    }
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
    fn should_parse_step(){
        let (_,k)=step(r#"print(text:@text"hello")"#.as_bytes()).unwrap();
        k.execute(&Environment::new_rc(MockProvider(vec![(format!("hobbies.size"),Value::Long(3)),
                                                         (format!("category"),Value::String(format!("Atmaram")))]))
        );
    }
    #[test]
    fn should_parse_block(){
        let (_,_)=journey(r#"abc{print(text:@text"hello");}"#.as_bytes()).unwrap();

    }

    #[test]
    fn should_parse_text_template_arg(){
        let (_,k)=text_template_arg(r#"@text"hello""#.as_bytes()).unwrap();
        println!("{:?}",k)
    }
    #[test]
    fn should_json_template_arg(){
        let (_,k)=json_template_arg(r#"@json{{abc}}"#.as_bytes()).unwrap();
        println!("{:?}",k)
    }

    #[test]
    fn should_parse_journey(){
        let (_,_)=journey(r#"`create nested categories`{
    for(outer:Object in outers){
        for(category:String in outer.var){
            post(url:@text"http://localhost:8080/api/category",
                body:@json{
                    "name":{{category}}
                },
                response:@nil);
        };
    };
}"#.as_bytes()).unwrap();
    }
    #[test]
    fn should_parse_journey_with_headers(){
        let (_,_)=journey(r#"`create nested categories`{
    for(outer:Object in outers){
        for(category:String in outer.var){
            post(url:@text"http://localhost:8080/api/category",
                body:@json{
                    "name":{{category}}
                },
                response:@nil,
                headers:@map{
                    "Content-Type":@text"application/json"
                }
                );
        };
    };
}"#.as_bytes()).unwrap();
    }
    #[test]
    #[test]
    fn test_get(){
        let (_,k)=journey(r#"`post for token`{
            get(url:@text"https://api.stripe.com/{{version}}/tokens",
                response:@ejson[<% for (id:Long in ids)
                                    {%>
                                                {
                                                    "id": {{id}}
                                                }
                                            <%}%>]);
        }"#.as_bytes()).unwrap();

    }

    #[test]
    fn test_delete(){
        let (_,k)=journey(r#"`get all ids`{
    delete(url:@text"http://localhost:8080/api/category/1",
                    response:@nil);
}"#.as_bytes()).unwrap();
    }

    #[test]
    fn check_parsing_text_argument(){
        let (_,k)=text_template_arg(r#"@text"<% for (id:Long in ids){%>
                                                             \{
                                                                 \"id\": {{id}}
                                                             }
                                                         <%}%>""#.as_bytes()).unwrap();
    }
    #[test]
    fn check_parsing_plain_text_argument(){
        let (_,k)=text_template_arg(r#"@text"http://localhost:8080/api/category""#.as_bytes()).unwrap();
    }
    #[test]
    fn check_parsing_json_argument(){
        let (_,k)=json_template_arg(r#"@json{"name":{{atmaram}}}"#.as_bytes()).unwrap();
    }
    #[test]
    fn check_parsing_ejson_argument(){
        let (_,k)=ejson_template_arg(r#"@ejson[<% for (id:Long in ids){%>
                                        {
                                            "id": {{id}}
                                        }
                                    <%}%>]"#.as_bytes()).unwrap();
        println!("{:?}",k)
    }
    #[test]
    fn test_stripe(){
        let (_,k)=journey(r#"`post for token`{
    post(url:@text"https://api.stripe.com/v1/tokens",
        body:@text"card[number]=5555555555554444&card[exp_month]=4&card[exp_year]=2021&card[cvc]=314&card[name]=atmaram+3@technogise.com",
                    headers:@map{
                    "Content-Type":@text"application/x-www-form-urlencoded"
                    },
                    response:@ejson[<% for (id:Long in ids){%>
                                        {
                                            "id": {{id}}
                                        }
                                    <%}%>]);
}"#.as_bytes()).unwrap();
        // k.execute(&Environment::new_rc(MockProvider(vec![])))
    }

}