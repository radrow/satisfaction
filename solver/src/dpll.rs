use crate::cnf::{
    CNF,
    CNFClause,
    CNFVar,
};
use std::fmt;
use std::collections::VecDeque;
use crate::{Solver, SATSolution, BranchingStrategy};
use crate::solvers::InterruptibleSolver;
use async_trait::async_trait;
use async_std::task::yield_now;

/// A DPLL based SAT-Solver, that solves a given SAT Problem. 
/// It needs a branching strategy for picking variables, the strategy has to be passed by calling 
/// the new method and specifying a datatype that implements the `trait BranchingStrategy`
/// 
/// # Example
/// ```
/// use solver::{SatisfactionSolver, NaiveBranching};
/// 
/// let solver = SatisfactionSolver::new(NaiveBranching);
/// let result = solver.solve(formula);
/// ```
pub struct SatisfactionSolver<B: BranchingStrategy> {
    strategy: B,
}

impl<B: BranchingStrategy> SatisfactionSolver<B> {
    pub fn new(strategy: B) -> SatisfactionSolver<B> {
        SatisfactionSolver {
            strategy
        }
    }
}

impl<B: BranchingStrategy> Solver for SatisfactionSolver<B> {
    fn solve(&self, formula: &CNF) -> SATSolution {
        let mut data = DataStructures::new(formula);
        data.dpll(&self.strategy)
    }
}

#[async_trait]
impl<B: BranchingStrategy+Send+Sync> InterruptibleSolver for SatisfactionSolver<B> {
    async fn solve_interruptible(&self, formula: &CNF) -> SATSolution {
        let mut data = DataStructures::new(formula);
        data.interruptible_dpll(&self.strategy).await
    }
}

/// Datatype for PrevAssignment, to store information if it was branching or unit propagation that
/// did the assignment.
/// 
/// # Values
/// 
/// * `Forced` - If variable was set during Unit-Propagation.
/// * `Branching` -  If variable was set while picking a new Variable for branching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentType {
    Forced, Branching
}


/// Datatype for assignment stack, which stores all assignments to variables that are done during the algorithm. 
/// Used to store assignments made in the past, to potentially undo them later with backtracking
/// 
/// # Attributes 
/// 
/// * `literal` - a literal as `CNFVar` that was assigned a value.
/// * `assignment_type` - the `AssignmentType` under which the variable was set.
struct PrevAssignment {
    literal: CNFVar,
    assignment_type: AssignmentType
}


/// The value of a variable
/// 
/// # Values
/// 
/// * `Pos` - Postive value equal true
/// * `Neg` - Negative value equal false
/// * `Free` - Variable has not been set yet
#[derive(Clone, Debug, PartialEq, Eq, Copy)]
pub enum VarValue {
    Pos, Neg, Free
}

impl std::ops::Neg for VarValue {
    type Output = VarValue;

    fn neg(self) -> Self::Output {
        match self {
            VarValue::Pos => VarValue::Neg,
            VarValue::Neg => VarValue::Pos,
            VarValue::Free => VarValue::Free,
        }
    }
}

impl From<bool> for VarValue {
    fn from(sign: bool) -> Self {
        match sign {
            true => VarValue::Pos,
            false => VarValue::Neg,
        }
    }
}

/// Variable Datatype, to store a variable.
/// Each variable has a value, which sets all occurrences of that variable to either true or 
/// false depending on the sign of variable in the clause.
/// 
/// # Attributes
/// 
/// * `value` - The value of the variable as `VarValue`
/// * `pos_occ` - describes the positive occurrences of a variable in a clause. The clause is referenced with index given in the vector.
/// * `neg_occ` - same as pos_occ, but for negated variables
pub struct Variable {
    pub value: VarValue,
    pub pos_occ: Vec<usize>,
    pub neg_occ: Vec<usize>,
}

/// Clause Datatype, to store a clause.
/// 
/// # Attributes
/// 
/// * `active_lit` - Describes the count of literals that haven't been set yet.
/// * `satisfied` - Is set if the clause has been satisfied by a variable, than with the index of the variable in the Variables Object, else it's None. 
/// * `literals` - Are all the variables that are in the clause as CNFVar.
pub struct Clause {
    pub active_lits: usize,
    pub satisfied: Option<usize>,
    pub literals: Vec<CNFVar>,
}

/// Datatype for all Variables, from a formula
pub type Variables = Vec<Variable>;
/// Datatype for all Clauses, from a formula
pub type Clauses = Vec<Clause>;


impl Variable {
    /// Method to create a new Variable-Object.
    /// Takes a CNF-Object that contains a CNF-Forumla and a the variable number of that 
    /// CNF-Formula. 
    /// 
    /// #Attributes
    /// 
    /// * `cnf` - A CNF-Object, which contains a CNF-Forumla
    /// * `var_num` - The variable number in the CNF-Object
    /// 
    /// # Example
    /// 
    /// ```
    /// // The formula `X and Y` would be converted into the integers `1 2`.
    /// let variable_1: Variable = Variable::new(cnf, 1);
    /// let variable_2: Variable = Variable::new(cnf, 2);
    /// ```
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

impl Clause {
    /// Method to create a new Clause-Object. 
    /// 
    /// # Arguments
    /// 
    /// * `cnf_clause` - A `CNFClause` that contains one clause of a formula for example
    /// from the CNF-Formula `(X and A) or (Y and Z)`, a clause would be `(X and A)`.
    fn new(cnf_clause: &CNFClause) -> Clause {
        // remove douplicated variables for active_lit, because they count as only 1 active literal
        let mut cnf_variables = cnf_clause.vars.clone();
        cnf_variables.sort();
        cnf_variables.dedup();
        cnf_variables.iter_mut()
            .for_each(|var| var.id -= 1);

        Clause {
            active_lits: cnf_variables.len(),
            satisfied: None,
            literals: cnf_variables,
        }
    }
}

/// Datatype for storing all internal objects used for the DPLL calculation
/// variables, all variables that the formula has.
/// # Attributes
/// * `clauses` - All clauses the forumla has
/// * `unit_queue` - Stores all unit-variables that have been found
/// * `assignment_stack` - All assignment that have been made so far
struct DataStructures {
    variables: Vec<Variable>,
    clauses: Vec<Clause>,
    unit_queue: VecDeque<CNFVar>,
    assignment_stack: Vec<PrevAssignment>,
}

impl DataStructures {
    /// The method to create a new DataStructure
    /// 
    /// #Attributes
    /// 
    /// * `cnf` - A CNF-Forumla
    fn new(cnf: &CNF) -> DataStructures {
        let clauses: Vec<Clause> = cnf.clauses.iter().map(|cnf_clause| Clause::new(&cnf_clause)).collect();
        let variables = (1..=cnf.num_variables).map(|i| Variable::new(&cnf, i)).collect();
        let unit_queue = VecDeque::with_capacity(cnf.num_variables);
        let assignment_stack = Vec::with_capacity(cnf.num_variables);

        DataStructures {
            variables,
            clauses,
            unit_queue,
            assignment_stack,
        }
    }

    async fn interruptible_dpll(&mut self, branching: &impl BranchingStrategy) -> SATSolution {
        // unit propagation
        if !self.inital_unit_propagation() {
            return SATSolution::Unsatisfiable;
        }

        // repeat & choose literal b
        while let Some(literal) = branching.pick_branching_variable(&self.variables, &self.clauses) {
            // set value b
            let conflict = !(self.set_variable(literal, AssignmentType::Branching)
                // unit propagation
                && self.unit_propagation()
                && self.pure_literal_elimination());

            // If backtracking does not help, formula is unsat.
            if conflict && !self.backtracking() {
                return SATSolution::Unsatisfiable;
            }

            if self.satisfaction_check() {
                break;
            }
            yield_now().await;
        }


        // output assignment
        self.variables.iter().map(|x| match x.value {
            VarValue::Pos => true,
            VarValue::Neg => false,
            _ => false
        }).collect()
    }

    /// Dpll-Algorithm for solving SAT-Problems in CNF form.
    /// 
    /// # Arguments
    /// 
    /// * `branching` - The Branching strategy for picking variables
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Forumla is a CNF-Datatype containing a SAT-Problem in CNF form
    /// let mut data = DataStructures::new(formula);
    /// let sat_solution = data.dpll(NaiveBranching);
    /// ```
    fn dpll(&mut self, branching: &impl BranchingStrategy) -> SATSolution {
        // unit propagation
        if !self.inital_unit_propagation() {
            return SATSolution::Unsatisfiable;
        }

        // repeat & choose literal b
        while let Some(literal) = branching.pick_branching_variable(&self.variables, &self.clauses) {
            // set value b
            let conflict = !(self.set_variable(literal, AssignmentType::Branching)
                // unit propagation
                && self.unit_propagation()
                && self.pure_literal_elimination());

            // If backtracking does not help, formula is unsat.
            if conflict && !self.backtracking(){
                return SATSolution::Unsatisfiable;
            }

            if self.satisfaction_check() {
                break;
            }
        }

        // output assignment
        self.variables.iter().map(|x| match x.value {
            VarValue::Pos => true,
            VarValue::Neg => false,
            _ => false
        }).collect()
    }

    /// The unit propagation that happens initally before a variable is picked for branching.
    /// Returns a boolean, depending on if a conflict was found in `unit_propagation()`.
    fn inital_unit_propagation(&mut self) -> bool {
        // find all unit clauses and enqueue the variables in the queue
        for i in 0..self.clauses.len() {
            if self.clauses[i].active_lits == 1 {
                let unit_literal = self.find_unit_variable(i);
                if !self.unit_queue.contains(&unit_literal) {
                    self.unit_queue.push_back(unit_literal);
                }
            }
        }
        self.unit_propagation()
    }

    /// Method for setting a variable. 
    /// After setting a variable this method marks clauses as satisfied or 
    /// decrements the active literal count, depending on if the set variable was 
    /// positive or negative. Returns a boolean depending on if everything was ok 
    /// or a conflict was detected. Conflicts occurre in this function, if a variable 
    /// should be set but there are no more active variables left to be set.
    /// 
    /// # Arguments
    /// 
    /// * `lit` - a `CNFVar` that is going to be set.
    /// * `assign_type` - The assignment type the variable in which the variable is going to be set. 
    /// 
    /// # Example
    /// 
    /// ```
    /// // sets the variable 1 to true while branching
    /// if !self.set_variable(CNFVar {id: 1, sign: true}, AssignmentType::Branching) {
    ///     // handle conflict
    /// }
    /// ```
    fn set_variable(&mut self, lit: CNFVar, assign_type: AssignmentType) -> bool {
        self.variables[lit.id].value = lit.sign.into();
        self.assignment_stack.push(PrevAssignment { literal: lit, assignment_type: assign_type});

        let mut pos_occ: &Vec<usize> = &self.variables[lit.id].pos_occ;
        let mut neg_occ: &Vec<usize> = &self.variables[lit.id].neg_occ;
        let clauses = &mut self.clauses;

        // if the sign of the variable is negative, invert pos and neg occurrences
        if self.variables[lit.id].value == VarValue::Neg {
            neg_occ = &self.variables[lit.id].pos_occ;
            pos_occ = &self.variables[lit.id].neg_occ;
        }

        // set all pos_occ to satisfied
        pos_occ.iter().for_each(|p_occ| {
            if clauses[*p_occ].satisfied == None {
                clauses[*p_occ].satisfied = Some(lit.id)
            }
        });

        let mut no_conflict = true;
        for u in 0..neg_occ.len() {
            let clause = &mut self.clauses[neg_occ[u]];
            clause.active_lits -= 1;
            if clause.satisfied.is_none() {
                if clause.active_lits == 1 {
                    // unit literal detected
                    let unit_literal = self.find_unit_variable(neg_occ[u]);
                    if !self.unit_queue.contains(&unit_literal) {
                        self.unit_queue.push_back(unit_literal);
                    }
                } else if clause.active_lits <= 0 {
                    // conflict
                    no_conflict = false;
                }
            }
        };
        no_conflict
    }

    /// Method for unit propagation. Works through a unit_queue that contains all the currently found
    /// unit-variables and sets them depending on their current sign. A unit-variable is a variable,
    /// that is the last unset variable in a clause and has to become true depending on its sign to
    /// satisfy its clause. For example in `!A or (B and C)` the clause `!A` is a unit-clause 
    /// and A has to be set to false or else the first clause would be false.
    /// This Method sets variables with `AssignmentType::Forced`.
    /// Returns a boolean-value depending if a conflict was found while using the Method
    /// `set_variable()`.
    fn unit_propagation(&mut self) -> bool {
        while let Some(var) = self.unit_queue.pop_front() {
            if !self.set_variable(var, AssignmentType::Forced) {
                return false;
            }
        }
        true
    }

    // Method to eliminate literals that only exist as positive value in the formula.
    // Retuns true if successful and no conflict was detected.
    fn pure_literal_elimination(&mut self) -> bool {
        let pure_literals = self.variables.iter()
            .enumerate()
            .filter_map(|(id, var)| {
                match var.value {
                    VarValue::Free if var.pos_occ.is_empty() =>
                        Some(CNFVar::new(id, false)),
                    VarValue::Free if var.neg_occ.is_empty() => 
                        Some(CNFVar::new(id, true)),
                    _ => None
                }
            }).collect::<Vec<_>>();
        pure_literals.into_iter()
            .all(|literal| self.set_variable(literal, AssignmentType::Branching))
    }


    /// Method to undo the last assignments from unit propagation until the last
    /// time it was branched, and takes the other branch. Returns true if backtracking was successful and
    /// the clauses and variables could be restored, false if backtracking was not possible and the 
    /// SAT-Problem is not satisfiable.
    fn backtracking(&mut self) -> bool {
        while let Some(assign) = self.assignment_stack.pop() {
            let mut pos_occ: &Vec<usize> = &self.variables[assign.literal.id].pos_occ;
            let mut neg_occ: &Vec<usize> = &self.variables[assign.literal.id].neg_occ;
            if self.variables[assign.literal.id].value == VarValue::Neg {
                neg_occ = &self.variables[assign.literal.id].pos_occ;
                pos_occ = &self.variables[assign.literal.id].neg_occ;
            }

            for i in 0..neg_occ.len() {
                let n_occ = neg_occ[i];
                self.clauses[n_occ].active_lits += 1;
            }
            for i in 0..pos_occ.len() {
                let p_occ = pos_occ[i];
                if let Some(cl) = self.clauses[p_occ].satisfied {
                    if cl == assign.literal.id {
                        self.clauses[p_occ].satisfied = None
                    }
                }
            }

            // empty queue
            self.unit_queue.clear();

            // take other branch if assignment was branching
            if assign.assignment_type == AssignmentType::Branching {
                if self.set_variable(-assign.literal, AssignmentType::Forced) {
                    // goto unit propagation
                    if self.unit_propagation() == false {
                        return self.backtracking();
                    }
                }
                return true
            }
            self.variables[assign.literal.id].value = VarValue::Free;
        }
        // unsatisfied
        false
    }

    /// Finds the unit-variable that is in the given unit-clauses.
    /// Returns a `CNFVar`, of a Variable, that is a unit.
    /// 
    /// # Arguments
    /// 
    /// * `clause` - The index of the clause in which a unit variable should be searched.
    fn find_unit_variable(&self, clause: usize) -> CNFVar {
        self.clauses[clause].literals.iter()
            .filter(|lit| self.variables[lit.id].value == VarValue::Free)
            .next()
            .expect("The only left literal cound not be found!")
            .clone()
    }

    /// Checks if all clauses have been satisfied. Returns true
    /// if all are satisfied and false if there are still some that are not true yet and still 
    /// worked on.
    fn satisfaction_check(&mut self) -> bool {
        let mut satisfied = true;
        self.clauses.iter().for_each(|clause| {
            if clause.satisfied == None {
                satisfied = false;
            }
        });
        satisfied
    }

    #[allow(dead_code)]
    fn print_data_structures(&self) {
        for var in self.variables.iter() {
            print!("{}", var);
        }
        for cl in self.clauses.iter() {
            print!("{}", cl);
        }
        println!("");
    }
}
