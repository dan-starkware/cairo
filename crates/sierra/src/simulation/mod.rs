use std::collections::HashMap;

use itertools::izip;
use thiserror::Error;

use self::mem_cell::MemCell;
use crate::edit_state::{put_results, take_args, EditStateError};
use crate::ids::{FunctionId, VarId};
use crate::program::{Program, Statement, StatementId};
use crate::program_registry::{ProgramRegistry, ProgramRegistryError};

pub mod core;
pub mod mem_cell;
#[cfg(test)]
mod test;

/// Error occurring while simulating an extension.
#[derive(Error, Debug, Eq, PartialEq)]
pub enum ExtensionSimulationError {
    #[error("Expected different number of arguments")]
    WrongNumberOfArgs,
    #[error("Expected a different memory layout")]
    MemoryLayoutMismatch,
}

/// Error occurring while simulating a program function.
#[derive(Error, Debug, Eq, PartialEq)]
pub enum SimulationError {
    #[error("error from the program registry")]
    ProgramRegistryError(#[from] ProgramRegistryError),
    #[error("error from editing a variable state")]
    EditStateError(EditStateError, StatementId),
    #[error("error from simulating an extension")]
    ExtensionSimulationError(ExtensionSimulationError, StatementId),
    #[error("could not find the function to call")]
    MissingFunction,
    #[error("jumped out of bounds during simulation")]
    StatementOutOfBounds(StatementId),
    #[error("unexpected number of arguments to function")]
    FunctionArgumentCountMismatch { function_id: FunctionId, expected: usize, actual: usize },
    #[error("identifiers left at function return")]
    FunctionDidNotConsumeAllArgs(FunctionId, StatementId),
}

/// Runs a function from the program with the given inputs.
pub fn run(
    program: &Program,
    entry_point: &FunctionId,
    inputs: Vec<Vec<MemCell>>,
) -> Result<Vec<Vec<MemCell>>, SimulationError> {
    let registry = ProgramRegistry::new(program)?;
    // TODO(orizi): use registry to get the function info when it is in the registry.
    let func = program
        .funcs
        .iter()
        .find(|func| &func.id == entry_point)
        .ok_or(SimulationError::MissingFunction)?;
    let mut current_statement_id = func.entry;
    if func.params.len() != inputs.len() {
        return Err(SimulationError::FunctionArgumentCountMismatch {
            function_id: func.id.clone(),
            expected: func.params.len(),
            actual: inputs.len(),
        });
    }
    let mut state = HashMap::<VarId, Vec<MemCell>>::from_iter(
        izip!(func.params.iter(), inputs.into_iter())
            .map(|(param, input)| (param.id.clone(), input)),
    );
    loop {
        let statement = program
            .get_statement(&current_statement_id)
            .ok_or(SimulationError::StatementOutOfBounds(current_statement_id))?;
        match statement {
            Statement::Return(ids) => {
                let (remaining, outputs) = take_args(state, ids.iter()).map_err(|error| {
                    SimulationError::EditStateError(error, current_statement_id)
                })?;
                return if remaining.is_empty() {
                    Ok(outputs)
                } else {
                    Err(SimulationError::FunctionDidNotConsumeAllArgs(
                        func.id.clone(),
                        current_statement_id,
                    ))
                };
            }
            Statement::Invocation(invocation) => {
                let (remaining, inputs) =
                    take_args(state, invocation.args.iter()).map_err(|error| {
                        SimulationError::EditStateError(error, current_statement_id)
                    })?;
                let extension = registry.get_extension(&invocation.extension_id)?;
                let (outputs, chosen_branch) =
                    core::simulate(extension, inputs).map_err(|error| {
                        SimulationError::ExtensionSimulationError(error, current_statement_id)
                    })?;
                let branch_info = &invocation.branches[chosen_branch];
                state =
                    put_results(remaining, izip!(branch_info.results.iter(), outputs.into_iter()))
                        .map_err(|error| {
                            SimulationError::EditStateError(error, current_statement_id)
                        })?;
                current_statement_id = current_statement_id.next(&branch_info.target);
            }
        }
    }
}