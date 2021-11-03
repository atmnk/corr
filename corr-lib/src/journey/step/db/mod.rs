use crate::template::{VariableReferenceName, Expression};
use crate::journey::Executable;
use crate::core::runtime::Context;
use async_trait::async_trait;
use tokio::task::JoinHandle;
use rdbc_async::sql::Driver;
use crate::core::Value;
use crate::template::object::extractable::{ExtractableObject};

pub mod parser;
#[derive(Debug, Clone,PartialEq)]
pub struct DefineConnectionStep {
    connection_name:VariableReferenceName,
    connection_string:Expression
}
#[derive(Debug, Clone,PartialEq)]
pub struct ExecuteStep {
    connection_name:VariableReferenceName,
    query:Expression,
    is_single:bool,
    value:Expression,
}
#[derive(Debug, Clone,PartialEq)]
pub struct QueryStep {
    connection_name:VariableReferenceName,
    query:Expression,
    value:Expression,
    data_extractor_template:ExtractableObject,
}
#[async_trait]
impl Executable for DefineConnectionStep{
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let cstr = self.connection_string.evaluate(&context).await;
        let connection =  rdbc_async_postgres::sql::Driver.connect(cstr.to_string().as_str()).await.unwrap();
        context.connection_store.define(self.connection_name.to_string(),connection).await;
        return vec![];

    }
}

#[async_trait]
impl Executable for QueryStep {
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let conn = context.connection_store.get(self.connection_name.clone()).await.unwrap();
        let connection = conn.lock().await;
        let query = self.query.evaluate(context).await.to_string();
        let data = self.value.evaluate(context).await;
        println!("{}",query);
        if let Value::Array(values) = data {
            println!("{}",query);
            let stm = connection.prepare(query.as_str()).await.unwrap();
                let vals:Vec<rdbc_async::sql::Value> = values.iter().filter_map(|val|{
                    val.to_sql_value()
                }).collect();
            let _res = stm.execute_query(vals).await.unwrap();
        };
        return vec![]
    }
}

#[async_trait]
impl Executable for ExecuteStep {
    async fn execute(&self, context: &Context) -> Vec<JoinHandle<bool>> {
        let conn = context.connection_store.get(self.connection_name.clone()).await.unwrap();
        let connection = conn.lock().await;
        let query = self.query.evaluate(context).await.to_string();
        let data = self.value.evaluate(context).await;
        println!("{}",query);
        if let Value::Array(values) = data {
            println!("{}",query);
            let stm = connection.prepare(query.as_str()).await.unwrap();
            if self.is_single {
                let vals:Vec<rdbc_async::sql::Value> = values.iter().filter_map(|val|{
                    val.to_sql_value()
                }).collect();
                match stm.execute_update(vals).await {
                    Ok(r)=>{
                        println!("{} rows affected",r)
                    },
                    Err(e)=>{
                        println!("{:?}",e)
                    }
                }
            } else {
                let vals_of_vals:Vec<Vec<rdbc_async::sql::Value>> = values.iter().map(|val|{
                    match val {
                        Value::Array(values)=>{
                            values.iter().filter_map(|val|{ val.to_sql_value() }).collect()
                        },
                        _=>panic!("Improper usage")
                    }
                }).collect();
                for vals in vals_of_vals {
                    match stm.execute_update(vals).await {
                        Ok(r)=>{
                            println!("{} rows affected",r)
                        },
                        Err(e)=>{
                            println!("{:?}",e)
                        }
                    }
                }

            }

        };
        return vec![]

    }
}