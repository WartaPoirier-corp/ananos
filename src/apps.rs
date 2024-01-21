use crate::vm::Expression::{self, *};
use crate::vm::Value::*;

pub fn calc<'a>() -> Expression<'a> {
    Application(
        "add", &[
            Litteral(&Number(19)),
            Application(
                "abs",
                &[Litteral(&Number(-19))]
            ),
        ],
    )
}
