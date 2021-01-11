use crate::cnf;

use cnf::CNF;
use cnf::CNFClause;
use cnf::CNFVar;
use std::fmt;
use std::collections::VecDeque;
use crate::{Solver, Assignment};

pub struct SatisfactionSolver;

impl Solver for SatisfactionSolver {
    fn solve(&self, clauses: impl Iterator<Item=CNFClause>, num_variables: usize) -> Option<Assignment> { 
        dpll(&clauses.collect(), num_variables)
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
    active_cl: usize,
    satisfied: Option<usize>,
    literals: Vec<isize>
}

type Variables = Vec<Variable>;
type Clauses = Vec<Clause>;

impl Variable {
    fn new(cnf: &CNF, var_num: usize) -> Variable {
        Variable {
            value: VarValue::Free,
            neg_occ: cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
                    if clause.vars.contains(&CNFVar::Neg(var_num)) {
                        Some(index)
                    } else {
                        None
                    }
                }).collect(),
            pos_occ: cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
                    if clause.vars.contains(&CNFVar::Pos(var_num as usize)) {
                        Some(index)
                    } else {
                        None
                    }
                }).collect()
        }
    }
}

impl Clause {
    fn new(cnf_clause: &CNFClause) -> Clause {
        Clause {
            active_cl: cnf_clause.vars.len(),
            satisfied: None,
            literals: cnf_clause.vars.iter().map(|var| {
                match var {
                    CNFVar::Pos(s) => *s as isize,
                    CNFVar::Neg(s) => -(*s as isize)
                }
            }).collect()
        }
    }
}

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "act: {}, sat: {:?}, lit: ", self.active_cl, self.satisfied);
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

pub fn dpll(cnf: &CNF, num_of_vars: usize) -> Option<Vec<bool>> {
    let (mut variables, mut clauses) = create_data_structures(cnf, num_of_vars);
    let mut unit_queue: VecDeque<usize> = VecDeque::new();
    let mut assignment_stack: Vec<PrevAssignment> = Vec::new();

    print_datastructures(&variables, &clauses);

    while let Some(i) = pick_branching_variable(&variables) {
        set_literal(i, &mut variables, &mut clauses, &mut assignment_stack, &mut unit_queue, AssignmentType::Branching)?;

        loop {
            match unit_queue.pop_front() {
                Some(var_index) => {
                    variables[var_index].value = VarValue::Pos;
                    assignment_stack.push(PrevAssignment {variable: var_index, assignment_type: AssignmentType::Forced});
                    set_literal(var_index, &mut variables, &mut clauses, &mut assignment_stack, &mut unit_queue, AssignmentType::Forced)?;
                },
                None => break
            }
        }
    }
    println!("variable results: ");
    variables.iter().for_each(|v| print!("{:?} ", v.value));
    println!("\n-----------");

    Some(variables.iter().map(|x| match x.value {
        VarValue::Pos => true,
        VarValue::Neg => false,
        _ => false
    }).collect())
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

fn unit_propagation(i: usize, variables: &mut Variables, clauses: &mut Clauses, unit_queue: &mut VecDeque<usize>, assign_stack: &mut Vec<PrevAssignment>) -> Option<PrevAssignment> {
    let mut pos_occ: &Vec<usize> = &variables[i].pos_occ;
    let mut neg_occ: &Vec<usize> = &variables[i].neg_occ;
    if variables[i].value == VarValue::Neg {
        neg_occ = &variables[i].pos_occ;
        pos_occ = &variables[i].neg_occ;
    }

    pos_occ.iter().for_each(|p_occ| {clauses[*p_occ].satisfied = Some(i)});
    for u in 0..neg_occ.len() {
        let n_occ = neg_occ[u];
        
        clauses[n_occ].active_cl -= 1;

        if clauses[n_occ].active_cl == 1 {
            let unit_var_index = find_unit_variable_index(&clauses[n_occ], &variables);
            unit_queue.push_back(unit_var_index);
        } else if clauses[n_occ].active_cl <= 0 {
            unit_queue.clear();
            return backtracking(assign_stack, variables, clauses);
        }
    };
    Some(PrevAssignment {variable: i, assignment_type: AssignmentType::Empty})
}

fn backtracking(assignment_stack: &mut Vec<PrevAssignment>, variables: &mut Variables, clauses: &mut Clauses) -> Option<PrevAssignment> {
    while let Some(assign) = assignment_stack.pop() {
        variables[assign.variable as usize].value = VarValue::Free;
        for i in 0..variables[assign.variable as usize].neg_occ.len() {
            let n_occ = variables[assign.variable as usize].neg_occ[i];
            clauses[n_occ].active_cl += 1;
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
    variable_index as usize
}