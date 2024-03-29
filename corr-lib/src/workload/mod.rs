pub mod parser;
#[derive( Clone,PartialEq,Debug)]
pub struct WorkLoad1{
    pub name:String,
    pub startup_journey:Option<String>,
    pub journey:String,
    pub concurrent_users:usize,
    pub duration:u128,
    pub per_user_rampup:u64,
}
#[derive( Clone,PartialEq,Debug)]
pub struct WorkLoad {
    pub name:String,
    pub scenarios: Vec<Scenario>,
    pub setup:Option<String>,
}
#[derive( Clone,PartialEq,Debug)]
pub enum Scenario{
    Closed(ModelScenario),
    Open(ModelScenario)
}
#[derive( Clone,PartialEq,Debug)]
pub struct ModelScenario {
    pub journey: String,
    pub stages:Vec<ModelStage>,
    pub force_stop:Option<u64>,
}
#[derive( Clone,PartialEq,Debug)]
pub struct ModelStage {
    pub target:u64,
    pub duration:u64,
}

