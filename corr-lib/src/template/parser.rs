use crate::parser::{Parsable, ParseResult, ws, identifier_part, function_name};
use crate::template::{Expression, VariableReferenceName, Assignable, BinaryOperator, Operator, UnaryPostOperator, FunctionCallChain, UnaryPreOperator};
use nom::combinator::{map, verify};
use crate::core::{Value, Variable};
use nom::sequence::{tuple};
use nom::branch::alt;
use nom::character::complete::char;
use nom::multi::{separated_list0, separated_list1, many1};
use crate::template::text::Text;
use crate::template::object::FillableObject;
use nom::bytes::complete::tag;
use crate::template::functions::{function_names};
use crate::journey::parser::parse_name;

impl Parsable for Assignable {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(Expression::parser,|expr|Assignable::Expression(expr)),
            map(Text::parser,|txt|Assignable::FillableText(txt))
            ))(input)
    }
}
impl Parsable for BinaryOperator {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(ws(tag("&&")),|_| BinaryOperator::And),
            map(ws(tag("||")),|_| BinaryOperator::Or),
            map(ws(tag("==")),|_| BinaryOperator::Equal),
            map(ws(tag(">=")),|_| BinaryOperator::GreaterThanEqual),
            map(ws(tag("<=")),|_| BinaryOperator::LessThanEqual),
            map(ws(tag(">")),|_| BinaryOperator::GreaterThan),
            map(ws(tag("<")),|_| BinaryOperator::LessThan),
            map(ws(tag("!=")),|_| BinaryOperator::NotEqual),
            map(ws(tag("+")),|_| BinaryOperator::Add),
            map(ws(tag("-")),|_| BinaryOperator::Subtract),
            map(ws(tag("*")),|_| BinaryOperator::Multiply),
            map(ws(tag("/")),|_| BinaryOperator::Divide),
            map(ws(tag("%")),|_| BinaryOperator::Mod),
         ))(input)
    }
}
impl Parsable for UnaryPostOperator {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            map(ws(tag("++")),|_| UnaryPostOperator::Increment),
            map(ws(tag("--")),|_| UnaryPostOperator::Decrement)
        ))(input)
    }
}

impl Parsable for UnaryPreOperator {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(ws(tag("!")),|_| UnaryPreOperator::Not)(input)
    }
}

impl Operator {
    fn expression_with_operator_parser<'a>(input: &'a str) -> ParseResult<'a, Expression> {
        alt((
            Operator::binary_expression_chain,
            Operator::non_binary_expression
            ))(input)

    }
    fn non_binary_expression<'a>(input: &'a str) -> ParseResult<'a, Expression> {
        alt((
            map(tuple((many1(ws(UnaryPreOperator::parser)),ws(non_operator_expression) )), |(ops,right)|{
                let mut exp = right.clone();
                for opt  in ops {
                    exp = Expression::Operator(Operator::UnaryPre(opt), vec![exp])
                }
                exp
            }),
            map(tuple((ws(non_operator_expression), many1(ws(UnaryPostOperator::parser)))), |(left,ops)|{
                let mut exp = left.clone();
                for opt  in ops {
                    exp = Expression::Operator(Operator::UnaryPost(opt), vec![exp])
                }
                exp
            }),
            non_operator_expression
        ))(input)

    }
    // fn pure_binary_expression<'a>(input: &'a str) -> ParseResult<'a, Expression> {
    //     map(tuple((ws(Operator::non_binary_expression), ws(BinaryOperator::parser), ws(Operator::non_binary_expression))), |(left,op,right)|{
    //             Expression::Operator(Operator::Binary(op),vec![left,right])
    //         })(input)
    //
    // }
    // fn binary_expression<'a>(input: &'a str) -> ParseResult<'a, Expression> {
    //     map(tuple((ws(Operator::binary_expression_chain), ws(BinaryOperator::parser), ws(Operator::non_binary_expression))), |(left,op,right)|{
    //             Expression::Operator(Operator::Binary(op),vec![left,right])
    //     })(input)
    //
    // }
    fn _expression(mut left:Vec<(Expression,BinaryOperator)>,exp:Expression)->Expression{
        if left.len() == 0{
            return exp
        } else if let Some((next_exp,op)) = left.pop(){
            return Expression::Operator(Operator::Binary(op), vec![Self::_expression(left,next_exp),exp]);
        } else {
            unimplemented!()
        }
    }
    fn expression_right(mut left:Expression,right:Vec<(BinaryOperator,Expression)>,)->Expression{
        for (o,e) in right{
            left = Expression::Operator(Operator::Binary(o.clone()),vec![left,e.clone()])
        }
        return left
    }
    fn binary_expression_chain<'a>(input: &'a str)-> ParseResult<'a, Expression>{
        // map(tuple((many0(tuple((ws(Operator::non_binary_expression),ws(BinaryOperator::parser)))),ws(Operator::non_binary_expression))),|(left,right)|{
        //     Operator::expression(left,right)
        // })(input)
        map(tuple((ws(Operator::non_binary_expression),many1(tuple((ws(BinaryOperator::parser),ws(Operator::non_binary_expression)))))),|(left,right)|{
            Operator::expression_right(left,right)
        })(input)
    }
    // fn binary_expression<'a>(input: &'a str) -> ParseResult<'a, Expression> {
    //     alt((
    //         Operator::pure_binary_expression,
    //         Operator::non_binary_expression,
    //         map(tuple((ws(Operator::binary_expression), ws(BinaryOperator::parser), ws(Operator::pure_binary_expression))), |(left,op,right)|{
    //             Expression::Operator(Operator::Binary(op),vec![left,right])
    //         }),
    //     ))(input)
    //
    // }
}
impl Parsable for Expression{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
       alt((operator_expression,non_operator_expression))(input)
    }
}
fn naked_function_expression<'a>(input: &'a str)-> ParseResult<'a, Expression>{
        map(tuple((ws(function_name),ws(tag("(")),separated_list0(tag(","),ws(Expression::parser)),ws(tag(")")))),|(func,_,args,_)|{Expression::Function(func.to_string(),args)})(input)
}
fn non_operator_expression<'a>(input: &'a str)-> ParseResult<'a, Expression>{
    alt((
            stack_expression,
            map(FillableObject::parser,|fo|Expression::FillableObject(Box::new(fo))),
            map(Value::parser,|val|Expression::Constant(val)),
            map(FunctionCallChain::parser,|fcc|{
                let mut exp = fcc.left.clone();
                for (func,args) in fcc.function_chain {
                    let mut new_args = args.clone();
                    new_args.insert(0,exp.clone());
                    exp = Expression::Function(func,new_args)
                }
                exp
            }),
            naked_function_expression,
            map(Variable::parser,|val|Expression::Variable(val.name,val.data_type)),
    ))(input)
}
fn non_function_call_expression<'a>(input: &'a str)-> ParseResult<'a, Expression>{
    alt((
            stack_expression,
            naked_function_expression,
            map(Value::parser,|val|Expression::Constant(val)),
            map(VariableReferenceName::parser,|val|Expression::Variable(val.to_string(),Option::None)),
    ))(input)
}
fn stack_expression<'a>(input: &'a str)-> ParseResult<'a, Expression>{
    map(tuple((ws(tag("(")),ws(Expression::parser),ws(tag(")")))),|(_,exp,_)|{exp})(input)
}
fn operator_expression<'a>(input: &'a str)-> ParseResult<'a, Expression>{
    Operator::expression_with_operator_parser(input)
}
impl VariableReferenceName {
    fn non_function_parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
            alt((map(tuple((separated_list1(ws(tag(".")),ws(identifier_part)),verify(tuple((ws(tag(".")),ws(identifier_part))),|(_,last)|{!function_names().contains(last)}))),|(first,(_,last))|{
                let mut parts:Vec<String> = first.iter().map(|i|i.to_string()).collect();
                parts.push(last.to_string());
                VariableReferenceName{
                parts,
            }}),
                 map(verify(identifier_part,|part|{!function_names().contains(&part)}),|part|{VariableReferenceName::from(part)})
            ))(input)
        // map(
        //     separated_list1(ws(char('.')),
        //                     map(identifier_part,|val|{val.to_string()})),|parts| { VariableReferenceName {parts}})(input)
    }
}
impl Parsable for VariableReferenceName {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(
            separated_list1(ws(char('.')), parse_name),|parts| { VariableReferenceName {parts}})(input)
    }
}


impl Parsable for FunctionCallChain {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((non_function_call_expression,many1(tuple((ws(tag(".")),ws(function_name),ws(tag("(")),separated_list0(ws(tag(",")),ws(Expression::parser)),ws(tag(")"))))))),|(e,calls)|{FunctionCallChain{
            left:e,
            function_chain:calls.iter().map(|(_,func,_,args,_)|{(func.to_string(), args.clone())}).collect()
        }})(input)
    }
}
#[cfg(test)]
mod tests{
    use crate::parser::util::{assert_if, assert_no_error};
    use crate::parser::Parsable;
    use crate::template::{Expression, VariableReferenceName, Assignable, Operator, BinaryOperator, FunctionCallChain};
    use crate::core::{Value};
    use crate::template::text::{Text, Block, Scriplet};
    use crate::template::object::{FillableObject, FillableMapObject};
    
    use crate::template::parser::{stack_expression, naked_function_expression, non_function_call_expression};

    #[tokio::test]
    async fn should_parse_variable_reference_name_when_keyword(){
        let txt = r#"obj.`status`"#;
        let a = VariableReferenceName::parser(txt);
        assert_if(txt,a,VariableReferenceName::from("obj.status"))
    }

    #[tokio::test]
    async fn should_parse_variable_reference_name_when_dot(){
        let txt = r#"obj.day"#;
        let a = VariableReferenceName::parser(txt);
        assert_if(txt,a,VariableReferenceName::from("obj.day"))
    }
    #[tokio::test]
    async fn should_parse_nested_stack_expression(){
        let txt = r#"((10 + 10)/ (20 +20))"#;
        let _a = stack_expression(txt).unwrap();
    }
    #[tokio::test]
    async fn should_parse_simple_stack_expression(){
        let txt = r#"(20 + 20)"#;
        let _a = stack_expression(txt).unwrap();
    }

    #[tokio::test]
    async fn should_parse_not_eq(){
        let txt = r#"20 != 20"#;
        let _a = Expression::parser(txt).unwrap();
    }

    #[tokio::test]
    async fn should_parse_not(){
        let txt = r#"!is_true"#;
        let _a = Expression::parser(txt).unwrap();
    }

    #[tokio::test]
    async fn should_parse_and(){
        let txt = r#"a && b"#;
        assert_if(txt,Expression::parser(txt),Expression::Operator(Operator::Binary(BinaryOperator::And),vec![
            Expression::Variable("a".to_string(),Option::None),
            Expression::Variable("b".to_string(),Option::None)
        ]));
    }

    #[tokio::test]
    async fn should_parse_or(){
        let txt = r#"a || b"#;
        let _a = Expression::parser(txt).unwrap();
    }
    #[tokio::test]
    async fn should_parse_chain(){
        let txt = r#"a && b && c"#;
        assert_if(txt,Expression::parser(txt),Expression::Operator(
            Operator::Binary(BinaryOperator::And),
            vec![Expression::Operator(Operator::Binary(BinaryOperator::And),
                                      vec![
                                          Expression::Variable("a".to_string(),Option::None),
                                          Expression::Variable("b".to_string(),Option::None)]),
            Expression::Variable("c".to_string(),Option::None)
        ]
        ))
    }

    #[tokio::test]
    async fn should_parse_assignable_when_expression(){
        let txt = r#"name"#;
        let a = Assignable::parser(txt);
        assert_if(txt,a,Assignable::Expression(Expression::Variable(format!("name"),Option::None)))
    }

    #[tokio::test]
    async fn should_parse_assignable_when_fillabletext(){
        let txt = r#"text `Hello <%name%>`"#;
        let a = Assignable::parser(txt);
        assert_if(txt,a,Assignable::FillableText(Text {
            blocks:vec![
                Block::Text(format!("Hello ")),
                Block::Scriplet(Scriplet::Expression(Expression::Variable(format!("name"),Option::None)))
            ]
        }))

    }

    #[tokio::test]
    async fn should_parse_fillableobject_expression(){
        let txt = r#"object {}"#;
        let a = Expression::parser(txt);
        let map:FillableMapObject = FillableMapObject::WithPairs(vec![]);
        assert_if(txt,a,Expression::FillableObject(Box::new(FillableObject::WithMap(map))));
    }

    #[tokio::test]
    async fn should_parse_assignable_when_fillableobject(){
        let txt = r#"object name"#;
        let a = Assignable::parser(txt);
        assert_if(txt,a,Assignable::Expression(Expression::FillableObject(Box::new(FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None))))))
    }

    // #[tokio::test]
    // async fn should_parse_assignable_when_fillableobject_with_operator(){
    //     let txt = r#"object name % 15"#;
    //     let a = Assignable::parser(txt);
    //     assert_if(txt,a,Assignable::FillableObject(FillableObject::WithExpression(Expression::Variable(format!("name"),Option::None))))
    // }

    #[test]
    fn should_parse_expression_when_constant(){
        let text=r#""Atmaram""#;
        let a=Expression::parser(text);
        assert_if(text,a,Expression::Constant(Value::String("Atmaram".to_string())))
    }

    #[test]
    fn should_parse_expression_when_operator(){
        let text=r#"100 + 15 / 10"#;
        let a=Expression::parser(text);
        assert_if(text,a,
                  Expression::Operator(
                      Operator::Binary(BinaryOperator::Divide),
                      vec![
                          Expression::Operator(
                              Operator::Binary(BinaryOperator::Add),
                              vec![
                                  Expression::Constant(Value::PositiveInteger(100)),
                                  Expression::Constant(Value::PositiveInteger(15))
                              ]
                          ),
                          Expression::Constant(Value::PositiveInteger(10))
                      ]
                  )
        )
    }

    #[test]
    fn should_parse_expression_when_function(){
        let text=r#"concat("Atmaram","Naik")"#;
        let a=Expression::parser(text);
        assert_if(text,a,Expression::Function("concat".to_string(),vec![Expression::Constant(Value::String("Atmaram".to_string())),Expression::Constant(Value::String("Naik".to_string()))]))
    }
    #[test]
    fn should_parse_expression_when_dot_function_on_variable(){
        let text=r#"name.contains("Naik")"#;
        let a=Expression::parser(text);
        assert_if(text,a,Expression::Function("contains".to_string(),vec![Expression::Variable(format!("name"),Option::None),Expression::Constant(Value::String("Naik".to_string()))]))
    }
    fn should_parse_expression_when_dot_function_on_fqn_variable(){
        let text=r#"person.name.len()"#;
        let a=Expression::parser(text);
        assert_if(text,a,Expression::Function("len".to_string(),vec![Expression::Variable(format!("person.name"),Option::None),Expression::Constant(Value::String("Naik".to_string()))]))
    }
    #[test]
    fn should_parse_expression_when_chain_dot_function_on_variable(){
        let text=r#"name.concat("Naik").contains("Naik")"#;
        let a=Expression::parser(text);
        assert_if(text,a,Expression::Function("contains".to_string(),vec![Expression::Function(
            "concat".to_string(),vec![
                Expression::Variable(format!("name"),Option::None),
                Expression::Constant(Value::String("Naik".to_string()))]),
                Expression::Constant(Value::String("Naik".to_string()))
            ])
        )

    }
    #[test]
    fn should_parse_function_call_chain(){
        let text=r#"concat("ATM").contains("Naik")"#;
        let a=FunctionCallChain::parser(text);
        assert_no_error(text,a)
    }
    #[test]
    fn should_parse_non_function_call_expression(){
        let text=r#"concat("ATM")"#;
        let a=non_function_call_expression(text);
        assert_no_error(text,a)
    }
    #[test]
    fn should_parse_expression_when_dot_function_on_expression(){
        let text=r#"concat("ATM","R").contains("Naik")"#;
        let a=Expression::parser(text);
        assert_if(text,a,Expression::Function("contains".to_string(),vec![
            Expression::Function(format!("concat"),vec![
                Expression::Constant(Value::String("ATM".to_string())),
                Expression::Constant(Value::String("R".to_string())),
            ]),
            Expression::Constant(Value::String("Naik".to_string()))
        ]))
    }
    #[test]
    fn should_parse_naked_function_expression(){
        let text=r#"concat("ATM","Naik")"#;
        let a=naked_function_expression(text);
        assert_if(text,a,Expression::Function("concat".to_string(),vec![
            Expression::Constant(Value::String("ATM".to_string())),
            Expression::Constant(Value::String("Naik".to_string())),
        ]))
    }
    #[test]
    fn should_parse_expression_when_variable(){
        let text=r#"name"#;
        let a=Expression::parser(text);
        assert_if(text,a,Expression::Variable("name".to_string(),Option::None))
    }

    #[test]
    fn should_parse_variable_reference_name(){
        let text=r#"place.name"#;
        let a=VariableReferenceName::parser(text);
        assert_if(text,a,VariableReferenceName{parts:vec!["place".to_string(),"name".to_string()]})
    }
    #[test]
    fn should_parse_partially_when_keyword_in_variable_reference_name(){
        let text=r#"place.name.print"#;
        let a=VariableReferenceName::parser(text);
        assert_if(text,a,VariableReferenceName{parts:vec!["place".to_string(),"name".to_string()]})
    }
}