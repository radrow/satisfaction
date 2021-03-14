use std::{cell::RefCell,rc::Rc};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use itertools::Itertools;

use crate::{CNF, CNFClause, CNFVar, SATSolution, Solver};
use crate::cdcl::deletion_strategies::ClauseDeletionStrategy;
use crate::cdcl::restart_policies::RestartPolicy;
use crate::solvers::{FlagWaiter, InterruptibleSolver};

use super::{
    branching_strategies::BranchingStrategy,
    preprocessors::{Preprocessor, PreprocessorFactory},
    clause::{Clause, ClauseId, Clauses},
    learning_schemes::LearningScheme,
    update::Update,
    util::{BuildHasher, HashSet, IndexMap},
    variable::{Assignment, AssignmentType, Variable, VariableId, Variables},
    abstract_factory::AbstractFactory,
};


pub struct CDCLSolver<B,BF,L,LF,C,CF,R,RF,PF>
where BF: AbstractFactory<Product=B>,
      LF: AbstractFactory<Product=L>,
      CF: AbstractFactory<Product=C>,
      RF: AbstractFactory<Product=R>,
      PF: PreprocessorFactory
{
    branching_strategy: BF,
    learning_scheme: LF,
    deletion_strategy: CF,
    restart_policy: RF,
    preprocessor: PF,
}

impl<B,BF,L,LF,C,CF,R,RF,PF> CDCLSolver<B,BF,L,LF,C,CF,R,RF,PF>
where B: BranchingStrategy,
      L: LearningScheme,
      C: ClauseDeletionStrategy,
      R: RestartPolicy,

      BF: AbstractFactory<Product=B>,
      LF: AbstractFactory<Product=L>,
      CF: AbstractFactory<Product=C>,
      RF: AbstractFactory<Product=R>,
      PF: PreprocessorFactory
{
    pub fn new(branching_strategy: BF, learning_scheme: LF, deletion_strategy: CF, restart_policy: RF, preprocessor: PF) -> CDCLSolver<B,BF,L,LF,C,CF,R,RF,PF> {
        CDCLSolver {
            branching_strategy,
            learning_scheme,
            deletion_strategy,
            restart_policy,
            preprocessor,
        }
    }
}

impl<B,BF,L,LF,C,CF,R,RF,PF> Solver for CDCLSolver<B,BF,L,LF,C,CF,R,RF,PF>
where B: 'static+BranchingStrategy,
      L: 'static+LearningScheme,
      C: 'static+ClauseDeletionStrategy,
      R: 'static+RestartPolicy,

      BF: AbstractFactory<Product=B>,
      LF: AbstractFactory<Product=L>,
      CF: AbstractFactory<Product=C>,
      RF: AbstractFactory<Product=R>,
      PF: PreprocessorFactory,
{
    fn solve(&self, formula: &CNF) -> SATSolution {
        let mut preprocessor = self.preprocessor.new();

        let formula = preprocessor.preprocess(formula.clone());

        let execution_state = ExecutionState::new(
            formula,
            &self.branching_strategy,
            &self.learning_scheme,
            &self.deletion_strategy,
            &self.restart_policy
            );
        match execution_state {
            Some(state) => preprocessor.restore(state.cdcl()),
            None => return SATSolution::Unsatisfiable
        }
    }
}

#[async_trait]
impl<B,BF,L,LF,C,CF,R,RF,PF> InterruptibleSolver for CDCLSolver<B,BF,L,LF,C,CF,R,RF,PF>
where B: 'static+Send+Sync+BranchingStrategy,
      L: 'static+Send+Sync+LearningScheme,
      C: 'static+Send+Sync+ClauseDeletionStrategy,
      R: 'static+Send+Sync+RestartPolicy,

      BF: Send+Sync+AbstractFactory<Product=B>,
      LF: Send+Sync+AbstractFactory<Product=L>,
      CF: Send+Sync+AbstractFactory<Product=C>,
      RF: Send+Sync+AbstractFactory<Product=R>,

      PF: Send+Sync+PreprocessorFactory,
{
    async fn solve_interruptible(&self, formula: &CNF) -> SATSolution {
        let mut preprocessor = self.preprocessor.new();
        let formula = preprocessor.preprocess(formula.clone());

        async_std::task::yield_now().await;

        let execution_state = ExecutionState::new(
            formula,
            &self.branching_strategy,
            &self.learning_scheme,
            &self.deletion_strategy,
            &self.restart_policy
            );
        FlagWaiter::start(move |flag| {
            match execution_state {
                Some(state) => preprocessor.restore(state.interruptible_cdcl(flag)),
                None => return SATSolution::Unsatisfiable
            }
        }).await
    }
}


fn find_unit_clauses(cnf: &mut CNF, unit_queue: &mut IndexMap<VariableId, (bool, ClauseId)>) -> bool{
    let mut no_error = true;
    // filter_map -> filter
    cnf.clauses = cnf.clauses.iter().enumerate().filter_map(|(i, clause)| {
        if clause.vars.len() == 1 {
            let variable: CNFVar = clause.vars[0];
            if unit_queue.contains_key(&(variable.id-1)) {
                if let Some(var) = unit_queue.get_key_value(&(variable.id-1)) {
                    if var.1.0 != variable.sign {
                        no_error = false;
                    }
                }
            }
            unit_queue.insert(variable.id-1, (variable.sign, i));
        }       
        if clause.vars.len() == 1 {
            None
        } else {
            Some(clause.clone())
        }
    }).collect();
    no_error 
}

fn order_formula(cnf: &CNF) -> CNF {
    let mut order_cnf: CNF = CNF {clauses: Vec::new(), num_variables: cnf.num_variables};
    for cnf_clause in &cnf.clauses {
        let mut cnf_variables = cnf_clause.vars.clone();
        cnf_variables.sort();
        cnf_variables.dedup();
        order_cnf.clauses.push(CNFClause {vars: cnf_variables});
    }
    order_cnf
}



struct ExecutionState<B,L,C,R> {
    clauses: Clauses,
    variables: Variables,
    variables_cache: Option<Variables>,
    branching_depth: usize,
    assignment_stack: Vec<VariableId>,
    unit_queue: IndexMap<VariableId, (bool, ClauseId)>,

    branching_strategy: Rc<RefCell<B>>,
    learning_scheme: Rc<RefCell<L>>,
    clause_deletion_strategy: Rc<RefCell<C>>,
    restart_policy: Rc<RefCell<R>>,

    updates: Vec<Rc<RefCell<dyn Update>>>,
}

unsafe impl<B,L,C,R> Sync for ExecutionState<B,L,C,R> {}
unsafe impl<B,L,C,R> Send for ExecutionState<B,L,C,R> {}

impl<B,L,C,R> ExecutionState<B,L,C,R>
where B: 'static+BranchingStrategy,
      L: 'static+LearningScheme,
      C: 'static+ClauseDeletionStrategy,
      R: 'static+RestartPolicy {

    fn new(
        formula: CNF,
        branching_strategy: &impl AbstractFactory<Product=B>,
        learning_scheme: &impl AbstractFactory<Product=L>,
        deletion_strategy: &impl AbstractFactory<Product=C>,
        restart_policy: &impl AbstractFactory<Product=R>,
        ) -> Option<ExecutionState<B, L, C, R>> {

        let mut unit_queue = IndexMap::with_capacity_and_hasher(formula.num_variables, BuildHasher::default());

        let mut ordered_cnf: CNF = order_formula(&formula);

        if !find_unit_clauses(&mut ordered_cnf, &mut unit_queue) {
            return None;
        }

        // TODO: Avoid cloning
        let variables = (1..=ordered_cnf.num_variables)
                .map(|index| Variable::new(&ordered_cnf, index))
                .collect();

        let clauses = ordered_cnf.clauses
                .iter()
                .map(|cnf_clause| Clause::new(cnf_clause))
                .collect();

        let assignment_stack= Vec::with_capacity(formula.num_variables);

        let branching_strategy = Rc::new(RefCell::new(branching_strategy.create(&clauses, &variables)));
        let learning_scheme = Rc::new(RefCell::new(learning_scheme.create(&clauses, &variables)));
        let clause_deletion_strategy = Rc::new(RefCell::new(deletion_strategy.create(&clauses, &variables)));
        let restart_policy = Rc::new(RefCell::new(restart_policy.create(&clauses, &variables)));

        Some(ExecutionState {
            clauses,
            variables,
            variables_cache: None,
            branching_depth: 0,
            unit_queue,
            assignment_stack,

            updates: vec![
                branching_strategy.clone(),
                learning_scheme.clone(),
                clause_deletion_strategy.clone(),
                restart_policy.clone(),
            ],

            branching_strategy,
            learning_scheme,
            clause_deletion_strategy,
            restart_policy,
        })
    }

    fn cdcl(mut self) -> SATSolution {
        if self.unit_propagation().is_some() {
            return SATSolution::Unsatisfiable;
        }

        while let Some(literal) = {
            let mut bs = self.branching_strategy.borrow_mut();
            match bs.pick_literal(&self.clauses, &self.variables) {
                None => None,
                Some(literal) =>
                    match self.variables_cache {
                        None => Some(literal),
                        Some(ref caches) =>
                            match caches[literal.id].assignment {
                                None => Some(literal),
                                Some(assg) => Some(CNFVar{sign: assg.sign, ..literal})
                            }
                    }
            }
        } {
            self.branching_depth += 1;
            // Try to set a variable or receive the conflict clause it was not possible
            if self
                .set_variable(literal, AssignmentType::Branching)
                // If there was no conflict, eliminate unit clauses
                .or(self.unit_propagation())
                // If there was a conflict backtrack; return true if backtracking failed
                .filter(|_| {
                    let should_restart = self.restart_policy
                        .borrow_mut()
                        .restart();
                    if should_restart { self.restart() }
                    !should_restart
                })
                .map_or(false, |conflict_clause| {
                    self.backtracking(conflict_clause)
                })
            { return SATSolution::Unsatisfiable; }

            let to_delete = self.clause_deletion_strategy.borrow_mut()
                .delete_clause(&self.clauses, &self.variables);

            for clause in to_delete {
                let w = self.clauses.remove(clause);

                self.variables[w.0.id].remove_watched_occ(clause);
                self.variables[w.1.id].remove_watched_occ(clause);
            }
        }

        SATSolution::Satisfiable(
            self.variables
                .into_iter()
                .map(|var| var.assignment.map(|a| a.sign).unwrap_or(false))
                .collect_vec(),
        )
    }

    fn interruptible_cdcl(mut self, flag: Arc<AtomicBool>) -> SATSolution {
        if self.inital_unit_propagation() {
            flag.store(true, Ordering::Relaxed);
            return SATSolution::Unsatisfiable;
        }

        while let Some(literal) = {
            let mut bs = self.branching_strategy.borrow_mut();
            bs.pick_literal(&self.clauses, &self.variables)
        } {
            self.branching_depth += 1;
            // Try to set a variable or receive the conflict clause it was not possible
            if self
                .set_variable(literal, AssignmentType::Branching)
                // If there was no conflict, eliminate unit clauses
                .or(self.unit_propagation())
                // If there was a conflict backtrack; return true if backtracking failed
                .filter(|_| {
                    let should_restart = self.restart_policy
                        .borrow_mut()
                        .restart();
                    if should_restart { self.restart() }
                    !should_restart
                })
                .map_or(false, |conflict_clause| {
                    self.backtracking(conflict_clause)
                })
            {
                flag.store(true, Ordering::Relaxed);
                return SATSolution::Unsatisfiable;
            }
            if flag.load(Ordering::Relaxed) { return SATSolution::Unknown }

            let to_delete = self.clause_deletion_strategy.borrow_mut()
                .delete_clause(&self.clauses, &self.variables);

            for clause in to_delete {
                let w = self.clauses.remove(clause);

                self.variables[w.0.id].remove_watched_occ(clause);
                self.variables[w.1.id].remove_watched_occ(clause);
            }
        }

        flag.store(true, Ordering::Relaxed);

        SATSolution::Satisfiable(
            self.variables
                .into_iter()
                .map(|var| var.assignment.map(|a| a.sign).unwrap_or(false))
                .collect_vec(),
        )
    }


    fn restart(&mut self) {
        self.variables_cache = Some(self.variables.to_vec());
        self.branching_depth = 0;
        self.unit_queue.clear();
        while let Some(id) = self.assignment_stack.pop() {
            let assignment = self.variables[id].assignment.take().unwrap();
            let literal = CNFVar::new(id, assignment.sign);
            self.updates.iter().for_each(|up| up.borrow_mut().on_unassign(literal, &self.clauses, &self.variables));
        }
    }

    fn set_variable(&mut self, literal: CNFVar, assign_type: AssignmentType) -> Option<ClauseId> {
        // set the variable and remember the assignment
        let assignment = Assignment {
            sign: literal.sign,
            branching_level: self.branching_depth,
            reason: assign_type
        };

        self.variables[literal.id].assignment = Some(assignment);
        self.assignment_stack.push(literal.id);


        if self.variables[literal.id].watched_occ.len() > 0 {
            // when a set literal is also watched find a new literal to be watched
            return self.find_new_watched(literal.id);
        }

        None
    }

    fn unit_propagation(&mut self) -> Option<ClauseId> {
        while let Some((id, (sign, reason))) = self.unit_queue.pop() {
            let empty_clause = self.set_variable(CNFVar::new(id, sign), AssignmentType::Forced(reason));
            if empty_clause.is_some() {
                return empty_clause;
            }
        }
        None
    }

    fn inital_unit_propagation(&mut self) -> bool {
        for i in 0..self.clauses.len() {
            if self.clauses[i].literals.len() > 0 
                && self.clauses[i].watched_literals[0] == self.clauses[i].watched_literals[1] {
                let literal = self.clauses[i].get_first_watched();
                if self.add_to_unit_queue(literal, i)
                    || self.unit_propagation().is_some() {
                        return true;
                }
            }
        }
        false
    }

    fn add_to_unit_queue(&mut self, literal: CNFVar, reason: ClauseId) -> bool {
        self.unit_queue.insert(literal.id, (literal.sign, reason))
            .map_or(false, |(sign, _)| sign != literal.sign)
    }

    fn add_clause(&mut self, literals: CNFClause) -> usize {
        let unit = literals.len() == 1;
        let index = self.clauses.push(literals);

        self.variables[self.clauses[index].get_first_watched().id].add_watched_occ(index);
        if !unit {
            self.variables[self.clauses[index].get_second_watched().id].add_watched_occ(index);
        }

        index
    }

    fn backtracking(
        &mut self,
        empty_clause: ClauseId,
    ) -> bool {
        if empty_clause >= self.clauses.len_formula() {
            self.updates.iter()
                .for_each(|up| up.borrow_mut().on_conflict(empty_clause, &self.clauses, &self.variables));
        }


        let (conflict_clause, assertion_literal, assertion_level) = match {
            let mut ls = self.learning_scheme.borrow_mut();
            ls.find_conflict_clause(empty_clause, self.branching_depth, &self.clauses, &self.variables)
        } {
            Some(t) => t,
            None => return true,
        };

        let index = self.add_clause(conflict_clause);
        self.updates.iter()
            .for_each(|up| up.borrow_mut().on_learn(index, &self.clauses, &self.variables));

        while let Some(id) = self.assignment_stack.pop() {
            match self.variables[id].assignment {
                Some(assign) if assign.branching_level > assertion_level => {
                    let literal = self.variables[id].assignment.take()
                        .map(|assign| CNFVar::new(id, assign.sign))
                        .unwrap();
                    self.updates.iter().for_each(|up| up.borrow_mut().on_unassign(literal, &self.clauses, &self.variables));
                },
                _ => {
                    self.assignment_stack.push(id);
                    break;
                },
            }
        }

        self.branching_depth = assertion_level;
        self.unit_queue.clear();
        self.unit_queue.insert(assertion_literal.id, (assertion_literal.sign, index));

        self.unit_propagation()
            .map_or(false, |empty_clause| self.backtracking(empty_clause))
    }

   fn find_new_watched(&mut self, var_index: usize) -> Option<ClauseId> {
        // needs to be cloned before cause watched_occ will change its size while loop is happening 
        // due to removing and adding new occ
        let watched_occ: HashSet<ClauseId> = self.variables[var_index].watched_occ.clone();

        'clauses: for clause_index in watched_occ {
            let watched_literals: (CNFVar, CNFVar) = self.clauses[clause_index].get_watched_lits();

            // index of the watched variable that has been assigned a value
            let mut move_index: usize = 0;
            if watched_literals.1.id == var_index {
                move_index = 1;
            }

            if self.check_watched_lit_satisfied(clause_index) {
                continue 'clauses;
            }

            let mut no_unassign: bool = true;
            // find new literal
            for lit_index in 0..self.clauses[clause_index].literals.len() {
                match self.variables[self.clauses[clause_index].literals[lit_index].id].assignment {
                    // Variable is assigned to a value already
                    Some(assign) => {
                        // check if clause is satisfied
                        if assign.sign == self.clauses[clause_index].literals[lit_index].sign {
                            // clause is satisfied there is no need to find a new literal
                            continue 'clauses;
                        } 
                    },

                    // Variable not assigned yet
                    None => {
                        // unassigned variable found
                        no_unassign = false;

                        if self.change_watched_lists(lit_index, clause_index, move_index, var_index) {
                            continue 'clauses;
                        }
                    }
                }
            }

            if no_unassign {
                // report conflict 
                return Some(clause_index);
            } else {
                // unassigned variable is the other watched literal which means it is a unit variable
                let literal = self.variables[watched_literals.0.id].assignment
                    .map_or(watched_literals.0, |_| watched_literals.1);

                if self.add_to_unit_queue(literal, clause_index) { return Some(clause_index) }
            } 
        }
        None
    }

    fn change_watched_lists(&mut self, lit_index: usize, clause_index: usize, move_index: usize, var_index: usize) -> bool {
        let clause: &mut Clause = &mut self.clauses[clause_index];
        let literals: &Vec<CNFVar> = &clause.literals;
        let watched_index: [usize; 2] = clause.watched_literals;


        if lit_index != watched_index[0] && lit_index != watched_index[1] {
            clause.watched_literals[move_index] = lit_index;
            self.variables[literals[lit_index].id].add_watched_occ(clause_index);
            self.variables[var_index].remove_watched_occ(clause_index);
            return true
        }
        false
    }

    /// Method to check if one of the watched literals that has an assinged value also satisfies the 
    /// Clause.
    fn check_watched_lit_satisfied(&self, clause_index: usize) -> bool {
        let watched_index: [usize; 2] = self.clauses[clause_index].watched_literals;
        let literals: &Vec<CNFVar> = &self.clauses[clause_index].literals;

        let first = match self.variables[literals[watched_index[0]].id].assignment {
            Some(assign) => assign.sign == literals[watched_index[0]].sign,
            None => false
        };
        let second = match self.variables[literals[watched_index[1]].id].assignment {
            Some(assign) => assign.sign == literals[watched_index[1]].sign,
            None => false
        };
        first || second
    }
}
