use std::str;
use nom::character::complete::{anychar, char, multispace0};
use nom::{IResult, InputTake, Compare, UnspecializedInput, InputTakeAtPosition, AsChar};
use nom::error::ParseError;
use nom::bytes::complete::{tag, is_not, escaped, escaped_transform};
use nom::branch::alt;
use nom::sequence::{tuple, terminated, delimited, preceded, separated_pair};
use nom::combinator::{map, opt};
use corr_core::runtime::{ValueProvider, Variable, VarType, Environment};
use nom::multi::many0;
use corr_templates::json::parser::parse;
use corr_rest::{PostStep, GetStep};
use corr_journeys::{Executable, Journey, LoopStep, PrintStep};
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
    Nil
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
    function_call))(i)
}
fn text_template_arg(i:&[u8])->IResult<&[u8],Argument> {
    let fun= tuple((tag("@text`"),map(escaped_transform(is_not(r#"\`"#), '\\', |i: &[u8]| alt((tag("`"),tag("\\")))(i)),
                                      |abc| str::from_utf8(&abc).unwrap().to_string()),tag("`")));
    let (i,(_,tmpl,_))=fun(i)?;
    return Ok((i,Argument::Text(corr_templates::text::parser::parse(&tmpl).unwrap())));
}
fn json_template_arg(i:&[u8])->IResult<&[u8],Argument> {
    let fun= tuple((tag("@json`"),map(escaped_transform(is_not(r#"\`"#), '\\', |i: &[u8]| alt((tag("`"),tag("\\")))(i)),
                                      |abc| str::from_utf8(&abc).unwrap().to_string()),tag("`")));
    let (i,(_,tmpl,_))=fun(i)?;

    return Ok((i,Argument::Json(corr_templates::json::parser::parse(&tmpl).unwrap())));
}
fn ejson_template_arg(i:&[u8])->IResult<&[u8],Argument> {
    let fun= tuple((tag("@ejson`"),map(escaped_transform(is_not(r#"\`"#), '\\', |i: &[u8]| alt((tag("`"),tag("\\")))(i)),
                                       |abc| str::from_utf8(&abc).unwrap().to_string()),tag("`")));
    let (i,(_,tmpl,_))=fun(i)?;

    return Ok((i,Argument::ExtractableJson(corr_templates::json::extractable::parser::parse(&tmpl).unwrap())));
}
fn nil_arg(i:&[u8])->IResult<&[u8],Argument> {
    let (i,_)=tag("@nil")(i)?;
    Ok((i,Argument::Nil))
}
fn arg(i:&[u8])->IResult<&[u8],Argument> {
    alt(
        (text_template_arg,
        json_template_arg,
         ejson_template_arg,
         nil_arg,

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
fn resolve_post(args:HashMap<String,Argument>)->Box<dyn Executable>{
    let body=match args.get(&format!("body")) {
        Option::Some(arg_val)=>{
            match arg_val {
                Argument::Json(val)=>{
                    val.clone()
                },
                _=>{unimplemented!()}
            }
        }
        _=>unimplemented!()

    };
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
    Box::new(PostStep{
        url,
        body,
        response
    })
}
fn resolve_get(args:HashMap<String,Argument>)->Box<dyn Executable>{
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
    Box::new(GetStep{
        url,
        response
    })
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
        "print"=>resolve_print(args),
        _=>unimplemented!()
    }
}
pub fn read_journey_from_file(mut file:File)->Journey{
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    let (i,val)=journey(contents.as_bytes()).unwrap();
    return val;
}
#[cfg(test)]
mod tests{
    use crate::parser::{identifier, name, journey, step, block_steps, block, text_template_arg, json_template_arg};
    use nom::AsBytes;
    use corr_core::runtime::{ValueProvider, Environment, Variable, Value};
    use corr_journeys::Executable;

    #[test]
    fn should_parse_identifier(){
        let (i,k)=name("`atmaram \\`naik`".as_bytes()).unwrap();
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

        fn load_ith_as(&mut self, i: usize, index_ref_var: Variable, list_ref_var: Variable) {
            unimplemented!()
        }

        fn save(&self, var: Variable, value: Value) {
            unimplemented!()
        }
    }
    #[derive(Debug)]
    struct MockProvider(Vec<(String,Value)>);

    #[test]
    fn should_parse_step(){
        let (i,k)=step(r#"print(text:@text`hello`)"#.as_bytes()).unwrap();
        k.execute(&Environment::new_rc(MockProvider(vec![(format!("hobbies.size"),Value::Long(3)),
                                                         (format!("category"),Value::String(format!("Atmaram")))]))
        );
    }
    #[test]
    fn should_parse_block(){
        let (i,j)=journey(r#"abc{print(text:@text`hello`);}"#.as_bytes()).unwrap();

    }

    #[test]
    fn should_parse_text_template_arg(){
        let (i,k)=text_template_arg(r#"@text`hello`"#.as_bytes()).unwrap();
        println!("{:?}",k)
    }
    #[test]
    fn should_json_template_arg(){
        let (i,k)=json_template_arg(r#"@json`{{abc}}`"#.as_bytes()).unwrap();
        println!("{:?}",k)
    }

    #[test]
    fn should_parse_journey(){
        let (i,k)=journey(r#"`create nested categories`{
    for(outer:Object in outers){
        for(category:String in outer.var){
            post(url:@text`http://localhost:8080/api/category`,
                body:@json`{
                    "name":{{category}}
                }`,
                response:@nil);
        };
    };
}"#.as_bytes()).unwrap();
    }
    #[test]
    fn test_get_all_ids(){
        let (i,k)=journey(r#"`get all ids`{
    get(url:@text`http://localhost:8080/api/category`,
                    response:@ejson`[ <% for (id:Long in ids){%>
                                        {
                                            "id": {{id}}
                                        }
                                    <%}%>]`);
    print(text:@text`<% for (id:Long in ids){%>
                                                             {
                                                                 "id": {{id}}
                                                             }
                                                         <%}%>]`);
}"#.as_bytes()).unwrap();
        k.execute(&Environment::new_rc(MockProvider(vec![])))
    }
}