mod cnf;

use cnf::CNF;
use cnf::CNFClause;
use cnf::CNFVar;
use std::fmt;
use std::collections::VecDeque;
use std::error::Error;

#[derive(Clone, Debug, PartialEq)]
enum VarValue {
    Pos, Neg, Free
}

#[derive(PartialEq)]
enum AssignmentType {
    Forced, Branching
}

struct Assignment {
    variable: usize,
    assignment_type: AssignmentType
}

struct Variable {
    value: VarValue, 
    pos_occ: Vec<usize>,
    neg_occ: Vec<usize>
}

struct Clause {
    active_cl: usize,
    satisfied: isize,
    literals: Vec<isize>
}

type Variables = Vec<Variable>;
type Clauses = Vec<Clause>;

impl Variable {
    fn new(cnf: &CNF, var_num: usize) -> Variable {
        Variable {
            value: VarValue::Free,
            neg_occ: cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
                    if clause.vars.contains(&CNFVar::Neg(var_num as u32)) {
                        Some(index)
                    } else {
                        None
                    }
                }).collect(),
            pos_occ: cnf.clauses.iter().enumerate().filter_map(|(index, clause)| {
                    if clause.vars.contains(&CNFVar::Pos(var_num as u32)) {
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
            satisfied: -1,
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
        write!(f, "act: {}, sat: {}, lit: ", self.active_cl, self.satisfied);
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
    let mut variables: Variables = Vec::new();
    let mut clauses: Clauses = Vec::new();

    cnf.clauses.iter().for_each(|cnf_clause| clauses.push(Clause::new(cnf_clause)));
    (1..=num_of_vars).for_each(|i| variables.push(Variable::new(cnf, i)));

    (variables, clauses)
}

fn dpll(cnf: &CNF, num_of_vars: usize) -> Result<Variables, Box<dyn Error>> {
    let (mut variables, mut clauses) = create_data_structures(&cnf, num_of_vars);
    let mut unit_queue: VecDeque<usize> = VecDeque::new();
    let mut assignment_stack: Vec<Assignment> = Vec::new();

    print_datastructures(&variables, &clauses);

    while let Some(i) = pick_branching_variable(&variables) {
        set_literal(i, &mut variables, &mut clauses, &mut assignment_stack, &mut unit_queue)?;

        loop {
            match unit_queue.pop_front() {
                Some(var_index) => {
                    variables[var_index].value = VarValue::Pos;
                    assignment_stack.push(Assignment {variable: var_index, assignment_type: AssignmentType::Forced});
                    if unit_propagation(var_index, &mut variables, &mut clauses, &mut unit_queue, &mut assignment_stack)?.assignment_type == AssignmentType::Branching {
                        //todo
                    }
                },
                None => break
            }
        }
    }

    Ok(variables)
}

// if function returns none there couldnt be any variable picked anymore
fn pick_branching_variable(variables: &Variables) -> Option<usize> {
    // add heuristiks to chose variables
    for (i, v) in variables.iter().enumerate() {
        if v.value == VarValue::Free {
            return Some(i);
        }
    }
    None
}

fn set_literal(i: usize, variables: &mut Variables, clauses: &mut Clauses, assignment_stack: &mut Vec<Assignment>, unit_queue: &mut VecDeque<usize>) -> Result<(), Box<dyn Error>> {
    variables[i].value = VarValue::Pos;
    assignment_stack.push(Assignment {variable: i, assignment_type: AssignmentType::Branching});
    if unit_propagation(i, variables, clauses, unit_queue, assignment_stack)?.assignment_type == AssignmentType::Branching {
        variables[i].value = VarValue::Neg;
        assignment_stack.push(Assignment {variable: i, assignment_type: AssignmentType::Forced});
        unit_propagation(i, variables, clauses, unit_queue, assignment_stack)?;
    }
    Ok(())
}

fn unit_propagation(i: usize, variables: &mut Variables, clauses: &mut Clauses, unit_queue: &mut VecDeque<usize>, assign_stack: &mut Vec<Assignment>) -> Result<Assignment, Box<dyn Error>>{
    if variables[i].value == VarValue::Pos {
        variables[i].pos_occ.iter().for_each(|p_occ| clauses[*p_occ].satisfied = i as isize);
        for u in 0..variables[i].neg_occ.len() {
            let n_occ = variables[i].neg_occ[u];
            
            clauses[n_occ].active_cl -= 1;

            if clauses[n_occ].active_cl == 1 {
                let unit_var_index = find_unit_variable_index(&clauses[n_occ], &variables)?;
                unit_queue.push_back(unit_var_index);
            } else if clauses[n_occ].active_cl <= 0 {
                unit_queue.clear();
                return backtracking(assign_stack, variables, clauses);
            }
        };
    } else if variables[i].value == VarValue::Neg {
        variables[i].neg_occ.iter().for_each(|n_occ| clauses[*n_occ].satisfied = i as isize);
        for u in 0..variables[i].pos_occ.len() {
            let p_occ = variables[i].pos_occ[u];
            
            clauses[p_occ].active_cl -= 1;

            if clauses[p_occ].active_cl == 1 {
                let unit_var_index = find_unit_variable_index(&clauses[p_occ], &variables)?;
                unit_queue.push_back(unit_var_index);
            } else if clauses[p_occ].active_cl <= 0 {
                unit_queue.clear();
                return backtracking(assign_stack, variables, clauses);
            }
        };
 
    }
    Ok(Assignment {variable: 0, assignment_type: AssignmentType::Forced})
}

fn backtracking(assignment_stack: &mut Vec<Assignment>, variables: &mut Variables, clauses: &mut Clauses) -> Result<Assignment, Box<dyn Error>> {
    while let Some(assign) = assignment_stack.pop() {
        variables[assign.variable as usize].value = VarValue::Free;
        for i in 0..variables[assign.variable as usize].neg_occ.len() {
            let n_occ = variables[assign.variable as usize].neg_occ[i];
            clauses[n_occ].active_cl += 1;
        }
        for i in 0..variables[assign.variable as usize].pos_occ.len() {
            let p_occ = variables[assign.variable as usize].pos_occ[i];
            if clauses[p_occ].satisfied == assign.variable as isize{
                clauses[p_occ].satisfied = -1
            }
        }
        if assign.assignment_type == AssignmentType::Branching {
            return Ok(assign)
        }
    }
    Err("unsat".into())
}

fn find_unit_variable_index(clause: &Clause, variables: &Variables) -> Result<usize, Box<dyn Error>> {
    let mut variable_index: isize = -1;
    for lit in &clause.literals {
        if variables[(lit.abs()-1) as usize].value == VarValue::Free {
            variable_index = lit.abs() - 1;
            break;
        }
    }
    if variable_index == -1 {
        return Err("clause did not contain a free variable".into())
    }
    Ok(variable_index as usize)
}

fn main() {
    let input: Vec<Vec<isize>> = vec![vec![-1, 2], vec![-1, 3], vec![-2, 4], vec![-3, -4, 5], vec![-4, -5, 6, 7], vec![4, 5, -6]];
    let mut cnf: CNF = CNF::new();

    for clause in input {
        let mut cnf_clause: CNFClause = CNFClause::new();
        for var in clause {
            if var > 0 {
                cnf_clause.push(CNFVar::Pos(var.abs() as u32));
            } else {
                cnf_clause.push(CNFVar::Neg(var.abs() as u32));
            }
        }
        cnf.push(cnf_clause);
    }
    match dpll(&cnf, 7) {
        Ok(variables) => variables.iter().for_each(|v| print!("{}", v)),
        Err(e) => print!("{}", e)
    }

}
