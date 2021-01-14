use crate::cnf;

use cnf::CNF;
use cnf::CNFClause;
use cnf::CNFVar;
use core::panic;
use std::fmt;
use std::collections::VecDeque;
use crate::{Solver, Assignment};

pub trait BranchingStrategy {
    fn pick_branching_variable(&mut self, variables: &Variables, clauses: &Clauses) -> Option<usize>;
}

pub struct SatisfactionSolver;

impl Solver for SatisfactionSolver {
    fn solve(&self, formula: CNF, num_variables: usize) -> Assignment {
        self.dpll(&formula, num_variables)
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
        write!(f, "act: {}, sat: {:?}, lit: ", self.active_lits, self.satisfied)?;
        self.literals.iter().for_each(|lit| {if let Err(e) = write!(f, "{} ", lit){println!("{}", e)}});
        write!(f, "\n")
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "val: {:?}, pos: ", self.value)?;
        self.pos_occ.iter().for_each(|var| {if let Err(e) = write!(f, "{} ", var){println!("{}", e)}});
        write!(f, "| neg: ")?;
        self.neg_occ.iter().for_each(|var| {if let Err(e) = write!(f, "{} ", var){println!("{}", e)}});
        write!(f, "\n")
    }
}

impl SatisfactionSolver {
    fn dpll(&self, cnf: &CNF, num_of_vars: usize) -> Assignment {
        let (mut variables, mut clauses) = self.create_data_structures(cnf, num_of_vars);
        let mut unit_queue: VecDeque<isize> = VecDeque::new();
        let mut assignment_stack: Vec<PrevAssignment> = Vec::new();

        if self.inital_unit_propagation(&mut variables, &mut clauses, &mut unit_queue, &mut assignment_stack) == false {
            return Assignment::Unsatisfiable;
        }

        // As long as there are variable left pick one of them.
        while let Some(i) = self.pick_branching_variable(&variables) {
            if self.satisfaction_check(&clauses) {
                break;
            }

            if self.set_literal(i, &mut variables, &mut clauses, &mut assignment_stack, &mut unit_queue, AssignmentType::Branching, VarValue::Pos).is_none() {
                return Assignment::Unsatisfiable;
            }

            if self.process_unit_queue(&mut unit_queue, &mut variables, &mut clauses, &mut assignment_stack) == false {
                return Assignment::Unsatisfiable;
            }
        }

        variables.iter().map(|x| match x.value {
            VarValue::Pos => true,
            VarValue::Neg => false,
            _ => false
        }).collect()
    }

    fn create_data_structures(&self, cnf: &CNF, num_of_vars: usize) -> (Variables, Clauses) {
        let clauses: Clauses = cnf.clauses.iter().map(|cnf_clause| Clause::new(&cnf_clause)).collect();
        let variables: Variables = (1..=num_of_vars).map(|i| Variable::new(cnf, i)).collect();

        (variables, clauses)
    }

    fn satisfaction_check(&self, clauses: &Clauses) -> bool {
        let mut satisfied = true;
        clauses.iter().for_each(|clause| {
            if clause.satisfied == None {
                satisfied = false;
            }
        });
        satisfied
    }

    /// Funtion that picks the next variable to be chosen for branching.
    /// Returns the index of the next variable, or None if there is no Variable to be picked
    fn pick_branching_variable(&self, variables: &Variables) -> Option<usize> {
        // TODO -> add heuristics to chose Variables
        for (i, v) in variables.iter().enumerate() {
            if v.value == VarValue::Free {
                return Some(i);
            }
        }
        None
    }

    fn set_literal(&self, i: usize, variables: &mut Variables, clauses: &mut Clauses, assignment_stack: &mut Vec<PrevAssignment>, unit_queue: &mut VecDeque<isize>, assgn_type: AssignmentType, sign: VarValue) -> Option<()> {
        variables[i].value = sign;
        assignment_stack.push(PrevAssignment {variable: i, assignment_type: assgn_type});
        let assignment = self.unit_propagation(i, variables, clauses, unit_queue, assignment_stack)?;
        if assignment.assignment_type == AssignmentType::Branching {
            variables[assignment.variable].value = VarValue::Neg;
            assignment_stack.push(PrevAssignment {variable: assignment.variable, assignment_type: AssignmentType::Forced});
            self.unit_propagation(assignment.variable, variables, clauses, unit_queue, assignment_stack)?;
        }
        Some(())
    }

    fn process_unit_queue(&self, unit_queue: &mut VecDeque<isize>, variables: &mut Variables, clauses: &mut Clauses, assign_stack: &mut Vec<PrevAssignment>) -> bool{
        loop {
            match unit_queue.pop_front() {
                Some(var) => {
                    let mut sign = VarValue::Pos;
                    if var < 0 {
                        sign = VarValue::Neg;
                    }
                    if self.set_literal((var.abs() - 1) as usize, variables, clauses, assign_stack, unit_queue, AssignmentType::Forced, sign).is_none() {
                        return false;
                    }
                },
                None => break
            }
        }
        true
    }

    fn inital_unit_propagation(&self, variables: &mut Variables, clauses: &mut Clauses, unit_queue: &mut VecDeque<isize>, assign_stack: &mut Vec<PrevAssignment>) -> bool {
        for i in 0..clauses.len() {
            if clauses[i].active_lits == 1 {
                let unit_var_index: isize = self.find_unit_variable(&clauses[i], &variables);
                unit_queue.push_back(unit_var_index);
            }
        }
        
        self.process_unit_queue(unit_queue, variables, clauses, assign_stack)
    }

    fn unit_propagation(&self, i: usize, variables: &mut Variables, clauses: &mut Clauses, unit_queue: &mut VecDeque<isize>, assign_stack: &mut Vec<PrevAssignment>) -> Option<PrevAssignment> {
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
                    let unit_var_index: isize = self.find_unit_variable(&clauses[n_occ], &variables);
                    unit_queue.push_back(unit_var_index);
                } else if clauses[n_occ].active_lits <= 0 {
                    unit_queue.clear();
                    return self.backtracking(assign_stack, variables, clauses);
                }
            }
        };
        Some(PrevAssignment {variable: i, assignment_type: AssignmentType::Empty})
    }

    fn backtracking(&self, assignment_stack: &mut Vec<PrevAssignment>, variables: &mut Variables, clauses: &mut Clauses) -> Option<PrevAssignment> {
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
        // unsatisfied
        None
    }

    fn find_unit_variable(&self, clause: &Clause, variables: &Variables) -> isize {
        let mut variable: isize = 0;
        for lit in &clause.literals {
            if variables[(lit.abs()-1) as usize].value == VarValue::Free {
                variable = *lit;
                break;
            }
        }
        if variable == 0 {
            panic!("no unit variable could be found!");
        }
        variable
    }
    
    #[allow(dead_code)]
    fn print_datastructures(&self, variables: &Variables, clauses: &Clauses) {
        for var in variables {
            print!("{}", var);
        }
        for cl in clauses {
            print!("{}", cl);
        }
        println!("");
    }
}