// use crate::{
//     errors::{FileLocation, RuntimeError},
//     parser::Term,
// };
use crate::{active_parser::ActiveParse, errors::RuntimeError};

// fn run_term() {}

// fn register_func(func: Term) -> Result<(), RuntimeError> {
//     todo!()
// }

// fn print(term: Term) -> Result<(), RuntimeError> {
//     todo!()
// }

// fn register_struct(_struct: Term) -> Result<(), RuntimeError> {
//     todo!()
// }

// fn run_block(block: Term) -> Result<(), RuntimeError> {
//     if let Term::Block { terms } = block {
//         for term in terms {
//             match term {
//                 Term::Block { .. } => {
//                     return Err(RuntimeError(
//                         "Cannot have directly nested block".to_string(),
//                         FileLocation::None,
//                     ))
//                 }
//                 Term::Func { .. } => register_func(term)?,
//                 Term::Print { .. } => todo!(),
//                 Term::DeclareVar { .. } => todo!(),
//                 Term::Return { .. } => todo!(),
//                 Term::UpdateVar { .. } => todo!(),
//                 Term::If { .. } => todo!(),
//                 Term::Loop { .. } => todo!(),
//                 Term::ReadLn { .. } => todo!(),
//                 Term::Break => todo!(),
//                 Term::Continue => todo!(),
//                 Term::Call { .. } => todo!(),
//                 Term::Struct { .. } => register_struct(term)?,
//             }
//         }
//     } else {
//         return Err(RuntimeError(
//             "Invalid Block Term".to_string(),
//             FileLocation::None,
//         ));
//     }

//     return Ok(());
// }

pub fn interpret(program: ActiveParse) -> Result<(), RuntimeError> {
    let _ = program;
    return Ok(());
}
