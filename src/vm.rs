//! Apps run in some kind of very basic VM for the moment,
//! making a true compiler is too much work.
//!
//! The VM actually just interprets some AST.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use alloc::string::String;

pub struct Context<'a>(BTreeMap<&'a str, Value<'a>>);

#[derive(Clone)]
pub struct Type(u64);

#[derive(Clone)]
pub enum Value<'a> {
    Ty(Type),
    Number(i64),
    Bool(bool),
    Function(&'a [&'a str], Expression<'a>),
    NativeFunction(fn(Vec<Value>) -> Result<Value<'a>, Error>),
}

#[derive(Clone)]
pub enum Expression<'a> {
    Application(&'a str, &'a [Expression<'a>]),
    Branch(&'a Expression<'a>, &'a Expression<'a>, &'a Expression<'a>),
    Litteral(&'a Value<'a>),
}

impl<'a> Value<'a> {
    pub fn to_string(&self) -> String {
        match self {
            Value::Ty(t) => format!("#{}", t.0),
            Value::Number(x) => format!("{}", x),
            Value::Bool(x) => if *x { "⊤".into() } else { "⊥".into() },
            Value::Function(_, _) => format!("<fun>"),
            Value::NativeFunction(_) => format!("<native fun>")
        }
    }
}

impl<'a> Expression<'a> {
    pub fn to_string(&self) -> String {
        match self {
            Expression::Branch(c, t, f) => format!(
                "si ({}) alors ({}) sinon ({})",
                c.to_string(),
                t.to_string(),
                f.to_string(),
            ),
            Expression::Litteral(v) => v.to_string(),
            Expression::Application(name, args) => if args.len() == 0 {
                format!("({})", name)
            } else {
                format!(
                    "({} {})",
                    name,
                    args.iter().map(Expression::to_string).collect::<Vec<_>>().join(" ")
                )
            },
        }
    }
}

#[derive(Debug)]
pub enum Error {
    TypeError(&'static str),
    UndefinedValue,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Context(BTreeMap::new())
    }

    pub fn insert(&mut self, k: &'a str, v: Value<'a>) {
        self.0.insert(k, v);
    }

    fn remove(&mut self, k: &'a str) {
        self.0.remove(k);
    }

    pub fn run<'b>(&mut self, expr: &'b Expression<'a>) -> Result<Value<'a>, Error> {
        match expr {
            Expression::Litteral(x) => Ok((*x).clone()),
            Expression::Application(name, ref args) => {
                let func = match self.0.get(name) {
                    Some(e) => e.clone(),
                    None => return Err(Error::UndefinedValue),
                };

                if args.is_empty() {
                    Ok(func)
                } else {
                    match func {
                        Value::Function(arg_names, body) => {
                            let mut arg_names = arg_names.iter();
                            for arg in *args {
                                let name = match arg_names.next() {
                                    Some(n) => n,
                                    None => return Err(Error::TypeError("Too much arguments")),
                                };
                                let val = self.run(arg)?;
                                self.insert(name, val);
                            }
                            if arg_names.next().is_some() {
                                return Err(Error::TypeError("Not enough arguments, partial applications are not yet supported"));
                            }

                            let res = self.run(&body);

                            for name in arg_names {
                                self.remove(name)
                            }

                            return res;
                        },
                        Value::NativeFunction(f) => {
                            let mut arg_values = Vec::with_capacity(args.len());
                            for arg in *args {
                                arg_values.push(self.run(arg)?);
                            }
                            f(arg_values)
                        }
                        _ => return Err(Error::TypeError("Tried to apply a constant")),
                    }
                }
            },
            Expression::Branch(cond, t, f) => {
                let c = match self.run(cond)? {
                    Value::Bool(c) => c,
                    _ => return Err(Error::TypeError("Conditions must be booleans")),
                };
                if c {
                    self.run(t)
                } else {
                    self.run(f)
                }
            },
        }
    }
}
