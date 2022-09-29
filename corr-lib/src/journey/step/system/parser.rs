use crate::parser::{Parsable, ws};
use crate::journey::step::system::{SystemStep, PrintStep, ForLoopStep, AssignmentStep, PushStep, ConditionalStep, IfPart, SyncStep, LoadAssignStep, JourneyStep, WaitStep, TransactionStep, MetricStep, WhileStep, ExitStep, RemoveStep};
use crate::parser::ParseResult;
use nom::combinator::{map, opt};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::bytes::complete::{tag};
use nom::character::complete::char;
use nom::branch::alt;
use crate::template::{VariableReferenceName, Assignable, Expression};
use crate::journey::step::Step;
use nom::multi::{many0, many1, separated_list0, separated_list1};
use crate::journey::parser::parse_executable_name;


impl Parsable for WhileStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("while")),ws(Expression::parser),delimited(ws(tag("{")),many1(ws(Step::parser)),ws(tag("}"))))),
        |(_,condition,steps)|WhileStep{condition,steps}
        )(input)
    }
}
impl Parsable for PrintStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((opt(ws(tag("debug"))),preceded(ws(tag("print")),ws(Assignable::parser)))),|(d,asg)|{PrintStep::WithAssignable(asg,d.map(|_|true).unwrap_or(false))})(input)
    }
}
impl Parsable for MetricStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("ingest")),ws(Expression::parser),ws(tag("as")),separated_list1(ws(tag(",")),ws(Expression::parser))))
            , |(_,value,_,tags)|MetricStep{value,tags}
        )(input)
    }
}
impl Parsable for TransactionStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("measure")),ws(Expression::parser),delimited(ws(tag("{")),many0(ws(Step::parser)),ws(tag("}"))))),
            |(_,name,block)|TransactionStep{
            block,
            name
        }
        )(input)
    }
}
impl Parsable for ForLoopStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
            for_left_part,
            ws(for_right_part))), |(on,(with,index,steps))|{ForLoopStep::WithVariableReference(on, with, index, steps)})(input)
    }
}
impl Parsable for AssignmentStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("let")),opt(ws(tag("global"))),ws(VariableReferenceName::parser),ws(char('=')),ws(Assignable::parser))),|(_,go,var,_,assbl)|{AssignmentStep::WithVariableName(var,assbl,go.is_some())})(input)
    }
}
impl Parsable for JourneyStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map( tuple((opt(terminated(ws(VariableReferenceName::parser),ws(tag(".")))),ws(parse_executable_name),ws(tag("(")),separated_list0(ws(tag(",")),Expression::parser),ws(tag(")")))),|(journey,en,_,args,_,)|{

            let mut jn = if let Some(jr)=journey{
                jr.parts.clone()
            } else {
                vec![]
            };
            jn.push(en);
            JourneyStep{
                journey:jn.join("."),
                args
            }
        })(input)
    }
}
impl Parsable for PushStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(VariableReferenceName::parser),ws(tag(".push")),ws(tag("(")),ws(Assignable::parser),ws(tag(")")))),|(var,_,_,assbl,_)|{PushStep::WithVariableName(var,assbl)})(input)
    }
}
impl Parsable for RemoveStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(VariableReferenceName::parser),ws(tag(".remove")),ws(tag("(")),ws(Expression::parser),ws(tag(")")))),|(var,_,_,assbl,_)|{RemoveStep::WithVariableName(var,assbl)})(input)
    }
}
fn for_left_part<'a>(input: &'a str) -> ParseResult<'a, VariableReferenceName>{
    map(tuple((
        ws(VariableReferenceName::parser),
        ws(char('.')),
        ws(tag("for"))
        )),|(vrn,_,_)|vrn)(input)
}
fn for_right_part<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>, Option<VariableReferenceName>, Vec<Step>)>{
    alt((
        arged_for_parser,
        unarged_for_parser

    ))(input)
}
fn unarged_for_parser<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>,Option<VariableReferenceName>,Vec<Step>)>{
    map(one_or_many_steps,|steps|{(Option::None,Option::None,steps)})(input)
}
fn one_or_many_steps<'a>(input: &'a str) -> ParseResult<'a, Vec<Step>>{
    alt((
        map(Step::parser,|step|{vec![step]}),
        preceded(ws(char('{')),terminated(many0(Step::parser),ws(char('}'))))
    ))(input)
}
fn arged_for_parser<'a>(input: &'a str) -> ParseResult<'a, (Option<VariableReferenceName>,Option<VariableReferenceName>,Vec<Step>)>{
    map(tuple((
        preceded(
            ws(char('(')),
            terminated(
            opt(
                tuple((
                    ws(VariableReferenceName::parser),
                    opt(preceded(ws(char(',')),VariableReferenceName::parser))))),ws(char(')')))),ws(tag("=>")),one_or_many_steps)),
        |(opt_vars,_,steps)|{
            let mut with = Option::None;
            let mut index= Option::None;
            if let Some(vars)=opt_vars{
                with = Option::Some(vars.0);
                index = vars.1
            }
            (with,index,steps)})(input)
}
impl Parsable for SystemStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            // map(preceded(tag("//"),is_not("\n\r")),|val:&str|SystemStep::Comment(val.to_string())),
            // map(delimited(tag("/*"), is_not("*/"), tag("*/")),|val:&str|SystemStep::Comment(val.to_string())),
            map(WaitStep::parser,|ws|{SystemStep::Wait(ws)}),
            map(ExitStep::parser,|ws|{SystemStep::Exit(ws)}),
            map(preceded(ws(tag("undef")),ws(VariableReferenceName::parser)),|vrn|{SystemStep::Undefine(vrn)}),
            map(TransactionStep::parser,|tr|{SystemStep::Transaction(tr)}),
            map(MetricStep::parser,|ms|{SystemStep::Metric(ms)}),
            map(preceded(ws(tag("background")),Step::parser),|step|{SystemStep::Background(vec![step])}),
            map(preceded(ws(tag("background")),delimited(ws(tag("{")),many0(ws(Step::parser)),ws(tag("}")))),|steps|{SystemStep::Background(steps)}),
            map(ConditionalStep::parser,|ps|{SystemStep::Condition(ps)}),
            map(WhileStep::parser,|ws|{SystemStep::While(ws)}),
            map(PrintStep::parser,|ps|{SystemStep::Print(ps)}),
            map(ForLoopStep::parser,|fls|{SystemStep::ForLoop(fls)}),
            map(LoadAssignStep::parser,|asst| SystemStep::LoadAssign(asst)),
            map(AssignmentStep::parser,|asst| SystemStep::Assignment(asst)),
            map(SyncStep::parser,|asst| SystemStep::Sync(asst)),
            map(PushStep::parser,|ps|{SystemStep::Push(ps)}),
            map(RemoveStep::parser,|rs|{SystemStep::Remove(rs)}),
            map(preceded(ws(tag("call")),ws(JourneyStep::parser)),|js|{SystemStep::JourneyStep(js)}),
            )
        )
            // map(tuple((ws(tag("let ")),ws(identifier),ws(char('=')),Expression::parser)),|(_,var,_,expr)|{SystemStep::Assign(var,expr)}),
        (input)
    }
}

impl Parsable for WaitStep {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("wait")),ws(Expression::parser))),|(_,wt)|{WaitStep::WithTime(wt)})(input)
    }
}
impl Parsable for ExitStep {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("exit")),ws(Expression::parser))),|(_,wt)|{ExitStep::WithCode(wt)})(input)
    }
}
impl Parsable for SyncStep {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("sync")),ws(VariableReferenceName::parser),opt(tuple((ws(tag("to")),ws(tag("sandbox")),Expression::parser))))),|(_,variable,sb)|{SyncStep{
            variable,
            sandbox:sb.map(|(_,_,e)|e)
        }})(input)
    }
}
impl Parsable for LoadAssignStep {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((ws(tag("let")),ws(VariableReferenceName::parser),ws(char('=')),ws(tag("load")),Expression::parser,opt(tuple((ws(tag("from")),ws(tag("sandbox")),Expression::parser))))),|(_,variable,_,_,default_value,sb)|{LoadAssignStep{
            variable,
            default_value,
            sandbox:sb.map(|(_,_,e)|e)
        }})(input)
    }
}
impl Parsable for ConditionalStep{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
            ws(tag("if")),
            Expression::parser, ws(tag("{")),
            many0(Step::parser),ws(tag("}")),
            many0(tuple((
                ws(tag("else")),
                ws(tag("if")),
                Expression::parser, ws(tag("{")),
                many0(Step::parser),ws(tag("}"))))),
            opt(tuple((ws(tag("else")), ws(tag("{")),
                       many0(Step::parser),ws(tag("}"))))))),
            |(_,fc,_,
                 fb, _,eip,ep)|{
                let mut if_parts = vec![];
                if_parts.push(IfPart{
                    condition:fc,
                    block:fb
                });
                let els:Vec<IfPart> = eip.iter().map(|(_,_,condition,_,block,_)|{
                    IfPart{
                        condition:condition.clone(),
                        block:block.clone()
                    }
                }).collect();
                if_parts.append(&mut els.clone());
                let else_part = ep.map(|(_,_,s,_)|s).unwrap_or(vec![]);
                Self{
                    if_parts,
                    else_part
                }
            })(input)
    }
}

#[cfg(test)]
mod tests{
    use crate::journey::step::system::{SystemStep, PrintStep, ForLoopStep, AssignmentStep, PushStep, JourneyStep};
    use crate::parser::Parsable;
    use crate::template::text::{Text, Block};
    use crate::parser::util::{assert_if, assert_no_error};
    use crate::template::{VariableReferenceName, Assignable, Expression};
    use crate::journey::step::Step;
    use crate::journey::step::system::parser::{one_or_many_steps, unarged_for_parser, for_right_part, for_left_part, arged_for_parser};

    #[tokio::test]
    async fn should_parse_for_left_part(){
        let j= r#"atmaram.naik.for"#;
        assert_if(j
                  ,for_left_part(j)
                  ,VariableReferenceName::from("atmaram.naik"))

    }

    #[tokio::test]
    async fn should_parse_for_right_without_args(){
        let j= r#"print text `Hello`;"#;
        assert_if(j
                  , for_right_part(j)
                  , (Option::None,Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false)))
            ]))

    }
    #[tokio::test]
    async fn should_parse_for_right_with_args(){
        let j= r#"( name , index ) => print text `Hello`;"#;
        assert_if(j
                  , for_right_part(j)
                  , (Option::Some(VariableReferenceName::from("name")),Option::Some(VariableReferenceName::from("index")),vec![
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false)))
            ]))

    }

    #[tokio::test]
    async fn should_parse_unarged_for(){
        let j= r#"print text `Hello`;"#;
        assert_if(j
                  ,unarged_for_parser(j)
                  ,(Option::None,Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false)))
            ]))

    }

    #[tokio::test]
    async fn should_parse_one_or_many_steps_when_one_step(){
        let j= r#"print text `Hello`;"#;
        assert_if(j
                  ,one_or_many_steps(j)
                  ,vec![
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false)))
            ])

    }

    #[tokio::test]
    async fn should_parse_one_or_many_steps_when_multiple_step(){
        let j= r#"{ print text `Hello`
            print text `Hello World`
        }"#;
        assert_if(j
                  ,one_or_many_steps(j)
                  ,vec![
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false))),
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello World".to_string())]}),false)))
            ])

    }

    #[tokio::test]
    async fn should_parse_arged_for_without_variables(){
        let j= r#"()=>print text `Hello`;"#;
        assert_if(j
                  ,arged_for_parser(j)
                  ,(Option::None,Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false)))
            ]))

    }

    #[tokio::test]
    async fn should_parse_arged_for_with_loop_variable(){
        let j= r#"(name)=>print text `Hello`;"#;
        assert_if(j
                  ,arged_for_parser(j)
                  ,(Option::Some(VariableReferenceName{parts:vec!["name".to_string()]}),Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false)))
            ]))

    }
    #[tokio::test]
    async fn should_parse_arged_for_with_loop_variable_and_index_variable(){
        let j= r#"(name,index)=>print text `Hello`;"#;
        assert_if(j
                  ,arged_for_parser(j)
                  ,(Option::Some(VariableReferenceName{parts:vec!["name".to_string()]}),Option::Some(VariableReferenceName{parts:vec!["index".to_string()]}),vec![
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false)))
            ]))

    }

    #[tokio::test]
    async fn should_parse_printstep(){
        let j= r#"print text `Hello`;"#;
        assert_if(j
                  ,PrintStep::parser(j)
                  ,PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false))

    }

    #[tokio::test]
    async fn should_parse_for_step(){
        let j= r#"atmaram.for print text `Hello`;"#;
        assert_if(j
                  ,ForLoopStep::parser(j)
                  ,ForLoopStep::WithVariableReference(VariableReferenceName{ parts:vec![format!("atmaram")]},Option::None,Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false)))
            ]))

    }

    #[tokio::test]
    async fn should_parse_assignment_step(){
        let j= r#"let a = name"#;
        assert_if(j
                  ,AssignmentStep::parser(j)
                  ,AssignmentStep::WithVariableName(VariableReferenceName::from("a"),Assignable::Expression(Expression::Variable(format!("name"),Option::None)),false))

    }
    #[tokio::test]
    async fn should_parse_push_step(){
        let j= r#"objects.push(obj)"#;
        assert_if(j
                  ,PushStep::parser(j)
                  ,PushStep::WithVariableName(VariableReferenceName::from("objects"),Assignable::Expression(Expression::Variable(format!("obj"),Option::None))))

    }




    #[tokio::test]
    async fn should_parse_systemstep_with_printstep(){
        let j= r#"print text `Hello`"#;
        assert_if(j
                  ,SystemStep::parser(j)
                  ,SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false)))

    }
    #[tokio::test]
    async fn should_parse_systemstep_with_push_step(){
        let j= r#"name.push(text `Hello`)"#;
        assert_if(j
                  ,SystemStep::parser(j)
                  ,SystemStep::Push(PushStep::WithVariableName(VariableReferenceName::from("name"),Assignable::FillableText(Text{
                blocks:vec![Block::Text(format!("Hello"))]
            }))))

    }
    #[tokio::test]
    async fn should_parse_systemstep_with_for_step(){
        let j= r#"atmaram.for print text `Hello`"#;
        assert_if(j
                  ,SystemStep::parser(j)
                  ,SystemStep::ForLoop(ForLoopStep::WithVariableReference(VariableReferenceName{ parts:vec![format!("atmaram")]},Option::None,Option::None,vec![
                Step::System(SystemStep::Print(PrintStep::WithAssignable(Assignable::FillableText(Text{blocks:vec![Block::Text("Hello".to_string())]}),false)))
            ])))

    }
    #[tokio::test]
    async fn should_parse_systemstep_with_assignment_step(){
        let j= r#"let name = text `Hello`"#;
        assert_if(j
                  ,SystemStep::parser(j)
                  ,SystemStep::Assignment(AssignmentStep::WithVariableName(VariableReferenceName::from("name"),Assignable::FillableText(Text{
                blocks:vec![Block::Text(format!("Hello"))]
            }),false)))

    }
    #[tokio::test]
    async fn should_parse_systemstep_with_global_assignment_step(){
        let j= r#"let global name = text `Hello`"#;
        assert_if(j
                  ,SystemStep::parser(j)
                  ,SystemStep::Assignment(AssignmentStep::WithVariableName(VariableReferenceName::from("name"),Assignable::FillableText(Text{
                blocks:vec![Block::Text(format!("Hello"))]
            }),true)))

    }
    #[tokio::test]
    async fn should_parse_systemstep_with_assignment_step_with_operator(){
        let j= r#"let obj.day = object index % 18"#;
        SystemStep::parser(j).unwrap();
    }
    #[tokio::test]
    async fn should_parse_if_step(){
        let j= r#"
            if 1==1 {
                print text `Hello`
            }
        "#;
        assert_no_error(j,SystemStep::parser(j))
    }
    #[tokio::test]
    async fn should_parse_journey_step_without_package(){
        let j= r#"Hello()"#;
        assert_if(j,JourneyStep::parser(j),JourneyStep{
            journey:format!("Hello"),
            args:vec![]
        })
    }
    #[tokio::test]
    async fn should_parse_journey_step_without_args(){
        let j= r#"com.qalense.Hello()"#;
        assert_if(j,JourneyStep::parser(j),JourneyStep{
            journey:format!("com.qalense.Hello"),
            args:vec![]
        })
    }
    #[tokio::test]
    async fn should_parse_journey_step_with_args(){
        let j= r#"com.qalense.Hello(a,b)"#;
        assert_if(j,JourneyStep::parser(j),JourneyStep{
            journey:format!("com.qalense.Hello"),
            args:vec![
                Expression::Variable(format!("a"),None),
                Expression::Variable(format!("b"),None)
            ]
        })
    }
    #[tokio::test]
    async fn should_parse_single_background_step(){
        let j= r#"
            background if 1==1 {
                print text `Hello`
            }
        "#;
        assert_no_error(j,SystemStep::parser(j))
    }
    #[tokio::test]
    async fn should_parse_multiple_background_steps(){
        let j= r#"
            background {
                print text `Hello`
            }
        "#;
        assert_no_error(j,SystemStep::parser(j))
    }
}
