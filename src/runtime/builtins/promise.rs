use crate::runtime::builtin::{BuiltinStruct, BuiltinMethod};
use crate::runtime::value::{Value, PromiseData, PromiseState};
use crate::runtime::{RuntimeError, RuntimeResult, Interpreter};
use loft_builtin_macros::loft_builtin;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

/// Wait for all promises to resolve and return an array of results
#[loft_builtin(Promise.all)]
fn promise_all(interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Ok(Value::Promise(Rc::new(RefCell::new(PromiseData {
            state: PromiseState::Resolved(Value::Array(Vec::new())),
        }))));
    }
    
    let promises = match &args[0] {
        Value::Array(arr) => arr,
        _ => return Err(RuntimeError::new("Promise.all() expects an array of promises")),
    };
    
    let mut results = Vec::new();
    for p in promises {
        match p {
            Value::Promise(data) => {
                let state = data.borrow().state.clone();
                match state {
                    PromiseState::Resolved(val) => results.push(val),
                    PromiseState::Rejected(val) => return Err(RuntimeError::new(format!("Promise.all() rejected: {:?}", val))),
                    PromiseState::Pending(task) => {
                        let res = interpreter.call_value(task, Vec::new())?;
                        data.borrow_mut().state = PromiseState::Resolved(res.clone());
                        results.push(res);
                    }
                }
            }
            _ => results.push(p.clone()), // Treat non-promises as resolved values
        }
    }
    
    Ok(Value::Promise(Rc::new(RefCell::new(PromiseData {
        state: PromiseState::Resolved(Value::Array(results)),
    }))))
}

/// Wait for the first promise to resolve. If all reject, it rejects.
#[loft_builtin(Promise.any)]
fn promise_any(interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("Promise.any() expects an array of promises"));
    }
    
    let promises = match &args[0] {
        Value::Array(arr) => arr,
        _ => return Err(RuntimeError::new("Promise.any() expects an array of promises")),
    };
    
    if promises.is_empty() {
        return Err(RuntimeError::new("Promise.any() called with empty array"));
    }
    
    let mut errors = Vec::new();
    for p in promises {
        match p {
            Value::Promise(data) => {
                let state = data.borrow().state.clone();
                match state {
                    PromiseState::Resolved(_) => return Ok(Value::Promise(data.clone())),
                    PromiseState::Rejected(val) => errors.push(val),
                    PromiseState::Pending(task) => {
                        match interpreter.call_value(task, Vec::new()) {
                            Ok(res) => {
                                data.borrow_mut().state = PromiseState::Resolved(res.clone());
                                return Ok(Value::Promise(data.clone()));
                            }
                            Err(e) => {
                                let err_val = Value::String(e.to_string());
                                data.borrow_mut().state = PromiseState::Rejected(err_val.clone());
                                errors.push(err_val);
                            }
                        }
                    }
                }
            }
            _ => return Ok(Value::Promise(Rc::new(RefCell::new(PromiseData {
                state: PromiseState::Resolved(p.clone()),
            })))),
        }
    }
    
    Err(RuntimeError::new(format!("All promises rejected in Promise.any(): {:?}", errors)))
}

/// Wait for all promises to settle (either resolve or reject)
#[loft_builtin(Promise.allSettled)]
fn promise_all_settled(interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Ok(Value::Promise(Rc::new(RefCell::new(PromiseData {
            state: PromiseState::Resolved(Value::Array(Vec::new())),
        }))));
    }
    
    let promises = match &args[0] {
        Value::Array(arr) => arr,
        _ => return Err(RuntimeError::new("Promise.allSettled() expects an array of promises")),
    };
    
    let mut results = Vec::new();
    for p in promises {
        let (status, value) = match p {
            Value::Promise(data) => {
                let state = data.borrow().state.clone();
                match state {
                    PromiseState::Resolved(val) => ("fulfilled".to_string(), val),
                    PromiseState::Rejected(val) => ("rejected".to_string(), val),
                    PromiseState::Pending(task) => {
                        match interpreter.call_value(task, Vec::new()) {
                            Ok(res) => {
                                data.borrow_mut().state = PromiseState::Resolved(res.clone());
                                ("fulfilled".to_string(), res)
                            }
                            Err(e) => {
                                let err_val = Value::String(e.to_string());
                                data.borrow_mut().state = PromiseState::Rejected(err_val.clone());
                                ("rejected".to_string(), err_val)
                            }
                        }
                    }
                }
            }
            _ => ("fulfilled".to_string(), p.clone()),
        };
        
        let mut outcome = HashMap::new();
        outcome.insert("status".to_string(), Value::String(status.clone()));
        if status == "fulfilled" {
            outcome.insert("value".to_string(), value);
        } else {
            outcome.insert("reason".to_string(), value);
        }
        
        results.push(Value::Struct {
            name: "PromiseSettledResult".to_string(),
            fields: outcome,
        });
    }
    
    Ok(Value::Promise(Rc::new(RefCell::new(PromiseData {
        state: PromiseState::Resolved(Value::Array(results)),
    }))))
}

/// Wait for the first promise to resolve
#[loft_builtin(Promise.race)]
fn promise_race(interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("Promise.race() expects an array of promises"));
    }
    
    let promises = match &args[0] {
        Value::Array(arr) => arr,
        _ => return Err(RuntimeError::new("Promise.race() expects an array of promises")),
    };
    
    if promises.is_empty() {
        return Ok(Value::Unit);
    }
    
    // Since our promises are eager and already resolved, race just returns the first one
    match &promises[0] {
        Value::Promise(data) => {
            let state = data.borrow().state.clone();
            match state {
                PromiseState::Pending(task) => {
                    let res = interpreter.call_value(task, Vec::new())?;
                    data.borrow_mut().state = PromiseState::Resolved(res.clone());
                    Ok(Value::Promise(Rc::new(RefCell::new(PromiseData {
                        state: PromiseState::Resolved(res),
                    }))))
                }
                _ => Ok(Value::Promise(data.clone())),
            }
        }
        _ => Ok(Value::Promise(Rc::new(RefCell::new(PromiseData {
            state: PromiseState::Resolved(promises[0].clone()),
        })))),
    }
}

/// Create a resolved promise
#[loft_builtin(Promise.resolve)]
fn promise_resolve(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let val = if args.is_empty() {
        Value::Unit
    } else {
        args[0].clone()
    };
    
    Ok(Value::Promise(Rc::new(RefCell::new(PromiseData {
        state: PromiseState::Resolved(val),
    }))))
}

/// Create a rejected promise (represented as a Promise holding an Err variant)
#[loft_builtin(Promise.reject)]
fn promise_reject(_interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    let val = if args.is_empty() {
        Value::Unit
    } else {
        args[0].clone()
    };
    
    // We'll wrap it in an EnumVariant "Err" if it's not already one
    let error_val = match &val {
        Value::EnumVariant { variant_name, .. } if variant_name == "Err" => val,
        _ => Value::EnumVariant {
            enum_name: "Result".to_string(),
            variant_name: "Err".to_string(),
            values: vec![val],
        },
    };
    
    Ok(Value::Promise(Rc::new(RefCell::new(PromiseData {
        state: PromiseState::Rejected(error_val),
    }))))
}

/// Spawn a task to run in the background
#[loft_builtin(Promise.spawn)]
fn promise_spawn(interpreter: &mut Interpreter, _this: &Value, args: &[Value]) -> RuntimeResult<Value> {
    if args.is_empty() {
        return Err(RuntimeError::new("Promise.spawn() requires a function or closure"));
    }
    
    let task = args[0].clone();
    let promise = Value::Promise(Rc::new(RefCell::new(PromiseData {
        state: PromiseState::Pending(task),
    })));
    interpreter.task_queue.push_back(promise.clone());
    
    Ok(promise)
}

pub fn create_promise_builtin() -> BuiltinStruct {
    let mut promise = BuiltinStruct::new("Promise");
    
    promise.add_method("all", promise_all as BuiltinMethod);
    promise.add_method("any", promise_any as BuiltinMethod);
    promise.add_method("allSettled", promise_all_settled as BuiltinMethod);
    promise.add_method("race", promise_race as BuiltinMethod);
    promise.add_method("resolve", promise_resolve as BuiltinMethod);
    promise.add_method("reject", promise_reject as BuiltinMethod);
    promise.add_method("spawn", promise_spawn as BuiltinMethod);
    
    promise
}

// Register the builtin automatically
crate::submit_builtin!("Promise", create_promise_builtin);
