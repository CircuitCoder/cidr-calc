mod utils;

use cidr_calculator::eval::Value;
use wasm_bindgen::prelude::*;
use cidr_calculator::parser::parse_single;
use cidr_calculator::eval::eval_stmt;
use cidr_calculator::eval::Scope;
use cidr_calculator::eval::format;

#[wasm_bindgen]
#[derive(Default)]
pub struct EvalState {
    scope: Scope
}

#[wasm_bindgen]
pub fn create_state() -> EvalState {
    Default::default()
}

#[wasm_bindgen]
pub fn print_scope(state: &EvalState) -> Vec<String> {
    state.scope.keys().map(|e| e.to_owned()).collect()
}

#[wasm_bindgen]
pub fn eval_input(state: &mut EvalState, input: String) -> Result<Vec<String>, String> {
    let stmt = parse_single(&input).map_err(|e| e.to_string())?;
    let (v, s) = eval_stmt(&stmt, state.scope.clone()).map_err(|e| e.to_string())?;
    state.scope = s;

    match v {
        Value::Unit => {
            Ok(vec![])
        }
        v => {
            Ok(format(&v).collect())
        }
    }
}
