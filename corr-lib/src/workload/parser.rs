use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, u64};
use nom::combinator::{map, opt};
use nom::multi::{separated_list0, separated_list1};
use nom::sequence::{delimited, tuple};
use crate::core::parser::string;
use crate::core::Variable;
use crate::journey::parser::parse_name;
use crate::parser::{Parsable, ParseResult, ws};
use crate::workload::{ModelScenario, ModelStage, Scenario, WorkLoad};

impl Parsable for WorkLoad {
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
                      parse_name,
                      ws(tag("(")),separated_list0(ws(tag(",")),Variable::parser),ws(tag(")")),ws(char('{')),
                      opt(delimited(tuple((ws(tag("startup")),ws(tag(":")))),ws(string), ws(tag(",")))),
                      ws(tag("scenarios")),ws(tag(":")),delimited(ws(tag("[")),separated_list1(ws(tag(",")),ws(Scenario::parser)),ws(tag("]"))))),
            |(name,_,_,_,_,setup,_,_,scenarios)| WorkLoad {
                setup,
                name,
            scenarios
        })(input)
    }
}
impl Parsable for Scenario{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        alt((
            open_model_scenario_parser,
            closed_model_scenario_parser
            ))(input)
    }
}
impl Parsable for ModelStage{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(delimited(ws(tag("{")),tuple((
            ws(u64),
            ws(tag(",")),
            ws(u64)
            )),ws(tag("}"))),|(target,_,duration)|ModelStage{
            target,
            duration
        })(input)
    }
}
impl Parsable for ModelScenario{
    fn parser<'a>(input: &'a str) -> ParseResult<'a, Self> {
        map(tuple((
                ws(tag("journey")),ws(tag(":")),ws(string),ws(tag(",")),
                ws(tag("stages")),ws(tag(":")),
                delimited(ws(tag("[")),separated_list1(ws(tag(",")),ws(ModelStage::parser)),ws(tag("]"))),
                opt(tuple((ws(tag(",")),ws(tag("forceStop")),ws(tag(":")),ws(u64))))
            )),|(_,_,journey,_,_,_,stages,fs)|
        ModelScenario{
            journey,
            stages,
            force_stop:fs.map(|(_,_,_,stop)|stop.clone())
        })(input)
    }
}
fn open_model_scenario_parser<'a>(input: &'a str) -> ParseResult<'a, Scenario> {
   map(delimited(ws(tag("{")),tuple((
       ws(tag("executor")),ws(tag(":")),ws(tag("\"open\"")),ws(tag(",")),
       ModelScenario::parser
       )),ws(tag("}"))),|(_,_,_,_,ms)|{Scenario::Open(ms)})(input)
}
fn closed_model_scenario_parser<'a>(input: &'a str) -> ParseResult<'a, Scenario> {
    map(delimited(ws(tag("{")),tuple((
        ws(tag("executor")),ws(tag(":")),ws(tag("\"closed\"")),ws(tag(",")),
        ModelScenario::parser
    )),ws(tag("}"))),|(_,_,_,_,ms)|{Scenario::Closed(ms)})(input)
}