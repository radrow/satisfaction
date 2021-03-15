use crate::{CNF, CNFVar, CNFClause, SATSolution};
use super::{preprocessor::Preprocessor, preprocessor::PreprocessorFactory};

pub struct SelfSubsumptionInstance;

impl Preprocessor for SelfSubsumptionInstance {
    fn preprocess(&mut self, cnf: CNF) -> CNF {
        let mut old_clauses = cnf.clauses.clone();
        for variable_number in 1..=cnf.num_variables {
            let mut pos_occ: Vec<CNFClause> = Vec::new();
            let mut neg_occ: Vec<CNFClause> = Vec::new();

            let neg_var = &CNFVar{id: variable_number, sign: false};
            let pos_var = &CNFVar{id: variable_number, sign: true};
            for clause in &old_clauses {
                if clause.vars.contains(pos_var) {
                    pos_occ.push(clause.clone());
                } else if clause.vars.contains(neg_var) {
                    neg_occ.push(clause.clone());
                } 
            }
            
            for pos_clause in &pos_occ {
                let old_neg = neg_occ.clone();
                for neg_clause in old_neg {
                    let no_var_neg_clause = CNFClause{ vars: neg_clause.vars
                        .clone()
                        .into_iter()
                        .filter(|var | var != neg_var)
                        .collect()
                    };
                    let no_var_pos_clause = CNFClause { vars: pos_clause.vars
                        .clone()
                        .into_iter()
                        .filter(|var| var != pos_var)
                        .collect()
                    };

                    if no_var_neg_clause.len() > 0 && no_var_pos_clause.len() > 0 {
                        if subsumption_test(&no_var_pos_clause, &no_var_neg_clause, cnf.num_variables) {
                            // add the clause that lost the literal
                            old_clauses.push(no_var_pos_clause.clone());
                            // remove the clause with the literal
                            old_clauses = old_clauses
                                .into_iter()
                                .filter(|clause| clause != pos_clause)
                                .collect();
                            break;
                        } else if subsumption_test(&no_var_neg_clause, &no_var_pos_clause, cnf.num_variables) {
                            // add the clause that lost the literal
                            old_clauses.push(no_var_neg_clause);
                            // remove the clause with the literal
                            old_clauses = old_clauses
                                .into_iter()
                                .filter(|clause| *clause != neg_clause)
                                .collect();
                            neg_occ = neg_occ
                                .into_iter()
                                .filter(|clause| *clause != neg_clause)
                                .collect();
                        }
                    }
                }
            }
        }

        CNF {
            num_variables: cnf.num_variables,
            clauses: old_clauses
        }

    }

    fn restore(&mut self, assignment: SATSolution) -> SATSolution {
        assignment
    }
}

fn sig(clause: &CNFClause, num_vars: usize, sign: bool) -> usize {
    let mut mask = 1;
    let mut result = 0;
    for i in 1..=num_vars {
        if clause.vars.contains(&CNFVar {id: i, sign}) {
            result = result | mask;
        }
        mask = mask << 1;
    }
    result
}

fn subsumption_test(clause1: &CNFClause, clause2: &CNFClause, num_vars: usize) -> bool {
    let pos_first = sig(clause1, num_vars, true);
    let pos_second = sig(clause2, num_vars, true);
    let neg_first = sig(clause1, num_vars, false);
    let neg_second = sig(clause2, num_vars, false);
    if  pos_first | pos_second == pos_first {
        if neg_first | neg_second == neg_first {
            return true;
        }
    }
    false
}

pub struct SelfSubsumption;

impl PreprocessorFactory for SelfSubsumption {
    fn new(&self) -> Box<dyn Preprocessor> {
        Box::new(SelfSubsumptionInstance {
        })
    }
}