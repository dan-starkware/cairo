#![allow(dead_code)]
#[cfg(test)]
#[path = "dup_and_ignore_test.rs"]
mod test;

use std::collections::{HashMap, HashSet};

use itertools::{chain, Itertools};
use sierra::ids::VarId;
use sierra::program::GenBranchTarget;
use utils::ordered_hash_set::OrderedHashSet;

use crate::pre_sierra::{LabelId, Statement};

/// Calculates the set of required existing variables per pre-Sierra statement.
pub fn calculate_required_vars_per_statement(
    next_statement_index_fetch: &NextStatementIndexFetch,
    required_vars: &mut Vec<OrderedHashSet<VarId>>,
    index: usize,
    statements: &[Statement],
) {
    // TODO(orizi): Use option instead of empty.
    if !required_vars[index].is_empty() {
        return;
    }
    required_vars[index] = match &statements[index] {
        // The required variables for an invocation, is all of its branches required vars, without
        // vars generated by the call, and its arguments.
        Statement::Sierra(sierra::program::GenStatement::Invocation(invocation)) => chain!(
            invocation.branches.iter().flat_map(|branch| {
                let next_index = next_statement_index_fetch.get(index, &branch.target);
                calculate_required_vars_per_statement(
                    next_statement_index_fetch,
                    required_vars,
                    next_index,
                    statements,
                );
                let results: HashSet<VarId> = branch.results.iter().cloned().collect();
                required_vars[next_index]
                    .iter()
                    .filter(move |v| !results.contains(v))
                    .cloned()
                    .collect_vec()
            }),
            invocation.args.iter().cloned()
        )
        .collect(),
        Statement::Sierra(sierra::program::GenStatement::Return(ret_vars)) => {
            // At return - the only required variables are the returned variables.
            ret_vars.iter().cloned().collect()
        }
        Statement::Label(_) => {
            // Labels are no-ops - so we just use the same as next line.
            calculate_required_vars_per_statement(
                next_statement_index_fetch,
                required_vars,
                index + 1,
                statements,
            );
            required_vars[index + 1].clone()
        }
    }
}

/// Helper to fetch the next statement index from a branch target.
pub struct NextStatementIndexFetch {
    label_to_statement: HashMap<LabelId, usize>,
}
impl NextStatementIndexFetch {
    pub fn new(statements: &[Statement]) -> Self {
        Self {
            label_to_statement: statements
                .iter()
                .enumerate()
                .filter_map(|(i, s)| match s {
                    Statement::Sierra(_) => None,
                    Statement::Label(label) => Some((label.id, i)),
                })
                .collect(),
        }
    }
    pub fn get(&self, index: usize, target: &GenBranchTarget<LabelId>) -> usize {
        match target {
            sierra::program::GenBranchTarget::Fallthrough => index + 1,
            sierra::program::GenBranchTarget::Statement(label) => {
                *self.label_to_statement.get(label).unwrap()
            }
        }
    }
}