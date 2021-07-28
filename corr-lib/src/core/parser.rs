use crate::parser::{Parsable, ParseResult, ws};
use crate::core::{Value, Variable, DataType};
use nom::combinator::{map, opt, value};
use nom::sequence::{tuple, preceded, delimited};
use nom::bytes::complete::{tag, escaped, is_not};
use nom::character::complete::{char, anychar, digit1};
use nom::branch::alt;
use crate::template::VariableReferenceName;

pub fn double<'a>(input: &'a str) -> ParseResult<'a, f64> {
    let mut num = tuple((opt(tag("-")),digit1,tag("."),digit1));
    let (i,(sign,char_nums,_,mant_num)) = num(input)?;
    let str_num=format!("{}.{}",char_nums,mant_num);
    let f_num=str_num.parse::<f64>().unwrap();
    if let Some(_)=sign{
        Ok((i,(f_num * -1.0)))
    } else {
        Ok((i,f_num))
    }
}
pub fn positive_integer<'a>(input: &'a str) -> ParseResult<'a, u128> {
    let (i,digits) = digit1(input)?;
    let str_num=format!("{}",digits);
    let f_num=str_num.parse::<u128>().unwrap();
    Ok((i,f_num))
}
pub fn integer<'a>(input: &'a str) -> ParseResult<'a, i128> {
    let mut num = tuple((opt(tag("-")),digit1));
    let (i,(sign,nums)) = num(input)?;
    let str_num=format!("{}",nums);
    let f_num=str_num.parse::<i128>().unwrap();
    if let Some(_)=sign{
        Ok((i,(f_num * -1)))
    } else {
        Ok((i,f_num))
    }
}
pub fn boolean<'a>(input: &'a str) -> ParseResult<'a, bool> {
    // This is a parser that returns `true` if it sees the string "true", and
    // an error otherwise
    let parse_true = value(true, tag("true"));

    // This is a parser that returns `true` if it sees the string "true", and
    // an error otherwise
    let parse_false = value(false, tag("false"));

    // `alt` combines the two parsers. It returns the result of the first
    // successful parser, or an error
    alt((parse_true, parse_false))(input)
}
pub fn string<'a>(input: &'a str) -> ParseResult<'a, String> {
    map(
        delimited(
            char('\"'),
            opt(escaped(is_not("\\\""), '\\', anychar)),
            char('\"'),
        ),
        |s:Option<&str>| s.map(|s| s.to_string()).unwrap_or("".to_string()),
    )(input)
}
impl Parsable for Value {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(tag("null"),|_|Value::Null),
            map(boolean,|val|Value::Boolean(val)),
            map(double,|val|Value::Double(val)),
            map(string,|val|Value::String(val)),
            map(positive_integer,|val|Value::PositiveInteger(val)),
            map(integer,|val|Value::Integer(val)),
            // map(tuple((
            //     ws(tag("array!")),
            //     ws(char('[')),
            //     separated_list0(ws(char(',')),ws(Value::parser)),
            //     ws(char(']'))
            //     )),|(_,_,values,_)|{
            //     Value::Array(values)
            // }),
            // map(tuple((
            //     ws(tag("object!")),
            //     ws(char('{')),
            //     separated_list0(ws(char(',')),tuple((
            //         ws(string),ws(char(':')), Value::parser
            //         ))),
            //     ws(char('}'))
            // )),|(_,_,pairs,_)|{
            //     let mut map = HashMap::new();
            //     for (key,_,value) in pairs  {
            //         map.insert(key,value);
            //     }
            //     Value::Map(map)
            // })
        ))(input)
    }
}
impl Parsable for Variable {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((VariableReferenceName::parser,opt(preceded(ws(char(':')),DataType::parser)))),|(vr_name,data_type)| Variable{name:vr_name.parts.join("."),data_type})(input)
    }
}
impl Parsable for DataType {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(tag("String"),|_|DataType::String),
            map(tag("Boolean"),|_|DataType::Boolean),
            map(tag("PositiveInteger"),|_|DataType::PositiveInteger),
            map(tag("Integer"),|_|DataType::Integer),
            map(tag("Double"),|_|DataType::Double),
            map(tag("Object"),|_|DataType::Object),
            map(tag("List"),|_|DataType::List),
            ))(input)
    }
}
#[cfg(test)]
mod tests{
    use crate::parser::Parsable;
    use crate::parser::util::assert_if;
    use crate::core::{Value};

    #[tokio::test]
    async fn should_parse_null_value(){
        let j= r#"null"#;
        assert_if(j
                  ,Value::parser(j)
                  ,Value::Null)

    }

    #[tokio::test]
    async fn should_parse_boolean_value(){
        let j= r#"true"#;
        assert_if(j
                  ,Value::parser(j)
                  ,Value::Boolean(true))

    }

    #[tokio::test]
    async fn should_parse_negative_double_value(){
        let j= r#"-12.33"#;
        assert_if(j
                  ,Value::parser(j)
                  ,Value::Double(-12.33))

    }
    #[tokio::test]
    async fn should_parse_positive_double_value(){
        let j= r#"12.00"#;
        assert_if(j
                  ,Value::parser(j)
                  ,Value::Double(12.00))

    }

    #[tokio::test]
    async fn should_parse_string_value(){
        let j= r#""Atmaram""#;
        assert_if(j
                  ,Value::parser(j)
                  ,Value::String("Atmaram".to_string()))

    }

    #[tokio::test]
    async fn should_parse_positive_integer_value(){
        let j= r#"12"#;
        assert_if(j
                  ,Value::parser(j)
                  ,Value::PositiveInteger(12))

    }

    #[tokio::test]
    async fn should_parse_positive_negative_integer(){
        let j= r#"-12"#;
        assert_if(j
                  ,Value::parser(j)
                  ,Value::Integer(-12))

    }

    // #[tokio::test]
    // async fn should_parse_array(){
    //     let j= r#"array!["Atmaram","Yogesh"]"#;
    //     assert_if(j
    //               ,Value::parser(j)
    //               ,Value::Array(vec![Value::String(format!("Atmaram")),Value::String(format!("Yogesh"))]))
    //
    // }
    // #[tokio::test]
    // async fn should_parse_object(){
    //     let j= r#"object!{"name":"Atmaram"}"#;
    //     let mut hm = HashMap::new();
    //     hm.insert(format!("name"),Value::String(format!("Atmaram")));
    //     assert_if(j
    //               ,Value::parser(j)
    //               ,Value::Map(hm));
    //
    // }
    // #[tokio::test]
    // async fn should_parse_object_in_array(){
    //     let mut hm1 = HashMap::new();
    //     hm1.insert(format!("name"),Value::String(format!("Clothes")));
    //
    //     let mut hm2 = HashMap::new();
    //     hm2.insert(format!("name"),Value::String(format!("Electronics")));
    //     let j= r#"array![
    //     object! {
    //         "name":"Clothes"
    //     },
    //     object! {
    //         "name":"Electronics"
    //     }
    // ]"#;
    //     assert_if(j
    //               ,Value::parser(j)
    //               ,Value::Array(vec![Value::Map(hm1),Value::Map(hm2)]))
    //
    // }
}
