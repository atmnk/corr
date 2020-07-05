extern crate postgres;
use corr_templates::text::Text;
use corr_journeys::Executable;
use corr_core::runtime::{Environment, Variable, Value};


use postgres::{Client, NoTls};
use corr_templates::text::parser::parse;
use corr_templates::Fillable;
use std::rc::Rc;
pub struct DBStep {
    pub connection:Text,
    pub query:Text
}
impl Executable for DBStep {
    fn execute(&self, runtime: &Environment) {
        let conn_str=self.connection.fill(runtime).to_string();
        println!("{}",conn_str);
        let mut client = &mut Client::connect(conn_str.as_str(), NoTls).unwrap();
            let filled= self.query.fill(runtime).to_string();
            println!("{}",filled);
            client.execute(
                filled.as_str()
            ,
            &[],
            ).unwrap();

    }
}

#[cfg(test)]
mod tests{
    use corr_core::runtime::{ValueProvider, Variable, Value, Environment, VarType};
    use crate::DBStep;
    use corr_journeys::Executable;
    use corr_templates::text::parser::parse;

    struct MockChannel;
    impl ValueProvider for MockChannel{

        fn save(&self, _var: Variable, _value: Value) {
            unimplemented!()
        }

        fn read(&mut self, _variable: Variable) -> Value {
            unimplemented!()
        }

        fn write(&mut self, text: String) {

        }

        fn set_index_ref(&mut self, _index_ref_var: Variable, _list_ref_var: Variable) {
            unimplemented!()
        }

        fn drop(&mut self, _str: String) {
            unimplemented!()
        }

        fn load_ith_as(&mut self, _i: usize, _index_ref_var: Variable, _list_ref_var: Variable) {
            unimplemented!()
        }

        fn load_value_as(&mut self, _ref_var: Variable, _val: Value) {
            unimplemented!()
        }

        fn close(&mut self) {

        }
    }
    #[test]
    fn should_run_db_query() {
        let step= DBStep {
            query:parse("SELECT 'hello'::TEXT").unwrap(),
            connection:parse("user=postgres host={{env(\"PG_HOST\",\"postgres\")}} port={{env(\"PG_PORT\",\"5432\")}}").unwrap(),
        };
        let mut runtime=Environment::new_rc(MockChannel);
        step.execute(&runtime);
    }
}