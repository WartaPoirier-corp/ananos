//! A framework to build apps. In the future, this will be implemented with system calls
//! and a real library that apps can link.

use crate::vm::{Context, Error, Expression, Value};
use alloc::vec::Vec;

pub fn setup(ctx: &mut Context) {
    ctx.insert("add", Value::NativeFunction(add));
    ctx.insert("sub", Value::NativeFunction(sub));
    ctx.insert("lt", Value::NativeFunction(lt));
    ctx.insert("abs", Value::Function(&["x"], Expression::Branch(
        &Expression::Application("lt", &[ Expression::Application("x", &[]), Expression::Litteral(&Value::Number(0)) ]),    
        &Expression::Application("sub", &[
            Expression::Litteral(&Value::Number(0)),
            Expression::Application("x", &[]),
        ]),
        &Expression::Application("x", &[]),
    )));
}

fn lt<'a>(args: Vec<Value>) -> Result<Value<'a>, Error> {
    if args.len() == 2 {
        match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
            _ => Err(Error::TypeError("Only numbers can be `lt`ed")),
        }
    } else {
        Err(Error::TypeError("`lt` takes 2 arguments"))
    }
}

fn add<'a>(args: Vec<Value>) -> Result<Value<'a>, Error> {
    if args.len() == 2 {
        match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            _ => Err(Error::TypeError("Only numbers can be `add`ed")),
        }
    } else {
        Err(Error::TypeError("`add` takes 2 arguments"))
    }
}

fn sub<'a>(args: Vec<Value>) -> Result<Value<'a>, Error> {
    if args.len() == 2 {
        match (&args[0], &args[1]) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(Error::TypeError("Only numbers can be `sub`ed")),
        }
    } else {
        Err(Error::TypeError("`sub` takes 2 arguments"))
    }
}
