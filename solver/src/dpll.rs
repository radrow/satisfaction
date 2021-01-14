use crate::cnf;

use cnf::CNF;
use cnf::CNFClause;
use cnf::CNFVar;
use std::fmt;
use std::collections::VecDeque;
use crate::{Solver, Assignment};

pub trait BranchingStrategy {
    fn pick_branching_variable(&mut self, variables: &Variables, clauses: &Clauses) -> Option<usize>;
}

pub struct SatisfactionSolver;

impl Solver for SatisfactionSolver {
    fn solve(&self, formula: CNF, num_variables: usize) -> Assignment {
        dpll(&formula, num_variables)
    }
}

#[derive(PartialEq)]
enum AssignmentType {
    Forced, Branching, Empty
}
/// Used to store assignments made in the past, for undoing them with backtracking
struct PrevAssignment {
    variable: usize,
    assignment_type: AssignmentType
}


#[derive(Clone, Debug, PartialEq)]
enum VarValue {
    Pos, Neg, Free
}

struct Variable {
    value: VarValue, 
    pos_occ: Vec<usize>,
    neg_occ: Vec<usize>
}

struct Clause {
    active_lits: usize,
    satisfied: Option<usize>,
    literals: Vec<isize>
}

type Variables = Vec<Variable>;
type Clauses = Vec<Clause>;

impl Variable {
    fn new(cnf: &CNF, var_num: usize) -> Variable {
        let mut v = Variable {
            value: VarValue::Free,
            neg_occ: cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
                    if clause.vars.contains(&CNFVar {id: var_num, sign: false}) {
                        Some(index)
                    } else {
                        None
                    }
                }).collect(),
            pos_occ: cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
                    if clause.vars.contains(&CNFVar {id: var_num, sign: true}) {
                        Some(index)
                    } else {
                        None
                    }
                }).collect()
        };
        // if variable is not used set it to false
        if v.neg_occ.len() == 0 && v.pos_occ.len() == 0 {
            v.value = VarValue::Neg;
        }
        v
    }
}

impl Clause {
    fn new(cnf_clause: &CNFClause) -> Clause {
        // remove douplicated variables for active_lit, because they count as only 1 active literal
        let mut cnf_variables = cnf_clause.vars.clone();
        cnf_variables.sort();
        cnf_variables.dedup();

        Clause {
            active_lits: cnf_variables.len(),
            satisfied: None,
            literals: cnf_clause.vars.iter().map(|var| {
                if var.sign {
                    return var.id as isize;
                } else {
                    return -1 * (var.id as isize);
                }
            }).collect()
        }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "act: {}, sat: {:?}, lit: ", self.active_lits, self.satisfied);
        self.literals.iter().for_each(|lit| {write!(f, "{} ", lit);});
        write!(f, "\n")
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "val: {:?}, pos: ", self.value);
        self.pos_occ.iter().for_each(|var| {write!(f, "{} ", var);});
        write!(f, "| neg: ");
        self.neg_occ.iter().for_each(|var| {write!(f, "{} ", var);});
        write!(f, "\n")
    }
}

fn print_datastructures(variables: &Variables, clauses: &Clauses) {
    for var in variables {
        print!("{}", var);
    }
    for cl in clauses {
        print!("{}", cl);
    }
    println!("");
}

fn create_data_structures(cnf: &CNF, num_of_vars: usize) -> (Variables, Clauses) {
    let clauses: Clauses = cnf.clauses.iter().map(|cnf_clause| Clause::new(&cnf_clause)).collect();
    let variables: Variables = (1..=num_of_vars).map(|i| Variable::new(cnf, i)).collect();

    (variables, clauses)
}

fn satisfaction_check(clauses: &Clauses) -> bool {
    let mut satisfied = true;
    clauses.iter().for_each(|clause| {
        if clause.satisfied == None {
            satisfied = false;
        }
    });
    satisfied
}

pub fn dpll(cnf: &CNF, num_of_vars: usize) -> Assignment {
    let (mut variables, mut clauses) = create_data_structures(cnf, num_of_vars);
    let mut unit_queue: VecDeque<usize> = VecDeque::new();
    let mut assignment_stack: Vec<PrevAssignment> = Vec::new();

    print_datastructures(&variables, &clauses);

    if inital_unit_propagation(&mut variables, &mut clauses, &mut unit_queue, &mut assignment_stack) == false {
        return Assignment::Unsatisfiable;
    }

    // As long as there are variable left pick one of them.
    while let Some(i) = pick_branching_variable(&variables) {
        if satisfaction_check(&clauses) {
            break;
        }

        if set_literal(i, &mut variables, &mut clauses, &mut assignment_stack, &mut unit_queue, AssignmentType::Branching).is_none() {
            return Assignment::Unsatisfiable;
        }

        if process_unit_queue(&mut unit_queue, &mut variables, &mut clauses, &mut assignment_stack) == false {
            return Assignment::Unsatisfiable;
        }
    }

    variables.iter().map(|x| match x.value {
        VarValue::Pos => true,
        VarValue::Neg => false,
        _ => false
    }).collect()
}

/// Funtion that picks the next variable to be chosen for branching.
/// Returns the index of the next variable, or None if there is no Variable to be picked
fn pick_branching_variable(variables: &Variables) -> Option<usize> {
    // TODO -> add heuristics to chose Variables
    for (i, v) in variables.iter().enumerate() {
        if v.value == VarValue::Free {
            return Some(i);
        }
    }
    None
}

fn set_literal(i: usize, variables: &mut Variables, clauses: &mut Clauses, assignment_stack: &mut Vec<PrevAssignment>, unit_queue: &mut VecDeque<usize>, assgn_type: AssignmentType) -> Option<()> {
    variables[i].value = VarValue::Pos;
    assignment_stack.push(PrevAssignment {variable: i, assignment_type: assgn_type});
    let assignment = unit_propagation(i, variables, clauses, unit_queue, assignment_stack)?;
    if assignment.assignment_type == AssignmentType::Branching {
        variables[assignment.variable].value = VarValue::Neg;
        assignment_stack.push(PrevAssignment {variable: assignment.variable, assignment_type: AssignmentType::Forced});
        unit_propagation(assignment.variable, variables, clauses, unit_queue, assignment_stack)?;
    }
    Some(())
}

fn process_unit_queue(unit_queue: &mut VecDeque<usize>, variables: &mut Variables, clauses: &mut Clauses, assign_stack: &mut Vec<PrevAssignment>) -> bool{
    loop {
        match unit_queue.pop_front() {
            Some(var_index) => {
                if set_literal(var_index, variables, clauses, assign_stack, unit_queue, AssignmentType::Forced).is_none() {
                    return false;
                }
            },
            None => break
        }
    }
    true
}

fn inital_unit_propagation(variables: &mut Variables, clauses: &mut Clauses, unit_queue: &mut VecDeque<usize>, assign_stack: &mut Vec<PrevAssignment>) -> bool {
    for i in 0..clauses.len() {
        if clauses[i].active_lits == 1 {
            let unit_var_index = find_unit_variable_index(&clauses[i], &variables);
            unit_queue.push_back(unit_var_index);
        }
    }
    
    process_unit_queue(unit_queue, variables, clauses, assign_stack)
}

fn unit_propagation(i: usize, variables: &mut Variables, clauses: &mut Clauses, unit_queue: &mut VecDeque<usize>, assign_stack: &mut Vec<PrevAssignment>) -> Option<PrevAssignment> {
    let mut pos_occ: &Vec<usize> = &variables[i].pos_occ;
    let mut neg_occ: &Vec<usize> = &variables[i].neg_occ;
    if variables[i].value == VarValue::Neg {
        neg_occ = &variables[i].pos_occ;
        pos_occ = &variables[i].neg_occ;
    }

    pos_occ.iter().for_each(|p_occ| {
        if clauses[*p_occ].satisfied == None {
            clauses[*p_occ].satisfied = Some(i)
        }
    });
    for u in 0..neg_occ.len() {
        let n_occ = neg_occ[u];
        if clauses[n_occ].satisfied == None {
            clauses[n_occ].active_lits -= 1;

            if clauses[n_occ].active_lits == 1 {
                let unit_var_index = find_unit_variable_index(&clauses[n_occ], &variables);
                unit_queue.push_back(unit_var_index);
            } else if clauses[n_occ].active_lits <= 0 {
                unit_queue.clear();
                return backtracking(assign_stack, variables, clauses);
            }
        }
    };
    Some(PrevAssignment {variable: i, assignment_type: AssignmentType::Empty})
}

fn backtracking(assignment_stack: &mut Vec<PrevAssignment>, variables: &mut Variables, clauses: &mut Clauses) -> Option<PrevAssignment> {
    while let Some(assign) = assignment_stack.pop() {
        variables[assign.variable as usize].value = VarValue::Free;
        for i in 0..variables[assign.variable as usize].neg_occ.len() {
            let n_occ = variables[assign.variable as usize].neg_occ[i];
            clauses[n_occ].active_lits += 1;
        }
        for i in 0..variables[assign.variable as usize].pos_occ.len() {
            let p_occ = variables[assign.variable as usize].pos_occ[i];
            if let Some(cl) = clauses[p_occ].satisfied {
                if cl == assign.variable {
                    clauses[p_occ].satisfied = None
                }
            }
        }
        if assign.assignment_type == AssignmentType::Branching {
            return Some(assign)
        }
    }
    // unsat
    None
}

fn find_unit_variable_index(clause: &Clause, variables: &Variables) -> usize {
    let mut variable_index: isize = -1;
    for lit in &clause.literals {
        if variables[(lit.abs()-1) as usize].value == VarValue::Free {
            variable_index = lit.abs() - 1;
            break;
        }
    }
    if variable_index == -1 {
        panic!("clause had only 1 more active variable, but no such active variable could be found!");
    }
    variable_index as usize
}
