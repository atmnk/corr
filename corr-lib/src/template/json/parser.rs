use crate::parser::{Parsable, ParseResult, ws};
use crate::template::json::{Json, Pair};
use nom::branch::alt;
use crate::template::Expression;
use nom::combinator::{map, opt};
use nom::sequence::{preceded, terminated, tuple};
use nom::character::complete::char;
use nom::multi::separated_list0;
use nom::bytes::complete::tag;
use crate::core::{Variable, Value};
use crate::core::parser::string;

impl Parsable for Pair{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            tuple((string,ws(char(':')),Json::parser)),|(key,_,value)|{
                Pair{
                    key,
                    value
                }
            }
        )(input)
    }
}
impl Parsable for Json{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(Value::parser,|val|{Json::Expression(Expression::Constant(val))}),
            map(preceded(ws(tag("{{")),terminated(Expression::parser,ws(tag("}}")))),|val|{Json::Expression(val)}),
            map(
                preceded(ws(char('[')),terminated(separated_list0(ws(char(',')),Json::parser),ws(char(']')))),
                |val|{Json::StaticArray(val)}
            ),
            map(
                tuple((
                    ws(tag("<%")),
                    ws(tag("for")),
                    ws(char('(')),
                    ws(Variable::parser),
                    ws(tag("in")),
                    ws(Variable::parser),
                    opt(preceded(ws(char(',')),Variable::parser)),
                    ws(char(')')),
                    ws(char('{')),
                    ws(tag("%>")),
                    ws(Json::parser),
                    ws(tag("<%")),
                    ws(char('}')),
                    ws(tag("%>")),
                    )),|(_,_,_,with,_,on,inxed_var,_,_,_,json,_,_,_)|{
                    Json::DynamicArray(with,on,Box::new(json),inxed_var)
                }
            ),
            map(preceded(ws(char('{')),terminated(separated_list0(ws(char(',')),Pair::parser),ws(char('}')))),|val|{Json::Object(val)})
            ))(input)
    }
}

#[cfg(test)]
mod tests{
    use crate::parser::util::{assert_if};
    use crate::template::json::{Json, Pair};
    use crate::parser::{Parsable};
    use crate::template::Expression;
    use crate::core::Value;

    // struct Reference {
    //     path:Vec<String>
    // }
    // struct Identifier {
    //     value:String
    // }
    // impl Parsable for Reference{
    //     fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
    //         map(separated_list1(char('.'),map(iden,|val|{val.to_string()})),|path| {Reference {path}})(input)
    //     }
    // }


    // fn try_this<'a>(input: &'a str) -> ParseResult<'a, Option<String>> {
    //     alt((
    //         map(tuple((Reference::parser,tag(".for"))),|(_,_)|{Option::None}),
    //         map(Reference::parser,|path|{Option::Some(format!("{}",path.path.join(".")))}),
    //         ))(input)
    //
    // }

    // #[test]
    // fn test_trial_parcel(){
    //     let text=r#"At_M.hello"#;
    //     let a=try_this(text);
    //     assert_if(text,a,Option::None)
    // }

    #[test]
    fn should_parse_pair(){
        let text=r#""name": "Atmaram""#;
        let a=Pair::parser(text);
        assert_if(text,a,Pair{
            key:"name".to_string(),
            value:Json::Expression(Expression::Constant(Value::String("Atmaram".to_string())))
        })
    }
    #[test]
    fn should_parse_plain_object(){
        let text=r#"{"name": "Atmaram"}"#;
        let a=Json::parser(text);
        assert_if(text,a,Json::Object(vec![Pair{
            key:"name".to_string(),
            value:Json::Expression(Expression::Constant(Value::String("Atmaram".to_string())))
        }]))
    }

}