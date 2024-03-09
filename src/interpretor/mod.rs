use crate::{errors::RuntimeError, parser::Term};

fn run_term() {}

pub fn interpret(program: Term) -> Result<(), RuntimeError> {
    match program {
        Term::Block { terms } => {
            for term in terms {
                run_term()
            }
            Ok(())
        }
        _ => Err(RuntimeError(
            "Invalid program parse received".to_string(),
            None,
        )),
    }
}
