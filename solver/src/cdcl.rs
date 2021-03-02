use itertools::Itertools;
use std::collections::{HashSet, VecDeque};

use crate::{CNFClause, CNFVar, SATSolution, Solver, CNF};

type VariableId = usize;
type ClauseId = usize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssignmentType {
    Forced(ClauseId),
    Branching,
}

#[derive(Debug, Clone, Copy)]
struct Assignment {
    sign: bool,
    branching_level: usize,
    reason: AssignmentType,
}

#[derive(Debug, Clone)]
struct Variable {
    pos_watched_occ: HashSet<ClauseId>,
    neg_watched_occ: HashSet<ClauseId>,
    assignment: Option<Assignment>,
}

struct Clause {
    literals: Vec<CNFVar>,
    watched_literals: [CNFVar; 2],
}

type Variables = Vec<Variable>;
type Clauses = Vec<Clause>;


trait BranchingStrategy {
    fn pick_literal(&self, clauses: &Clauses, variables: &Variables) -> Option<CNFVar>;
}

trait LearningScheme {
    fn find_conflict_clause(&self, conflict_clause: ClauseId, branching_depth: usize, clauses: &Clauses, variables: &Variables) -> (Clause, usize);
}

trait ClauseDeletionStrategy {
    fn delete_clause(&self, clauses: &mut Clauses, variables: &mut Variables);
}

struct ExecutionState {
    clauses: Clauses,
    variables: Variables,
    branching_depth: usize,
    unit_queue: VecDeque<CNFVar>,
}

impl ExecutionState {
    fn cdcl(
        mut self,
        branching_strategy: &impl BranchingStrategy,
        learning_scheme: &impl LearningScheme,
        clause_deletion_strategy: &impl ClauseDeletionStrategy,
    ) -> SATSolution {
        if self.unit_propagation().is_some() {
            return SATSolution::Unsatisfiable;
        }

        while let Some(literal) = branching_strategy.pick_literal(&self.clauses, &self.variables) {
            // Try to set a variable or receive the conflict clause it was not possible
            if self
                .set_variable(literal, false)
                // If there was no conflict, eliminate unit clauses
                .or(self.unit_propagation())
                // If there was a conflict backtrack; return true if backtracking failed
                .map_or(false, |conflict_clause| {
                    self.backtracking(conflict_clause, learning_scheme)
                })
            {
                return SATSolution::Unsatisfiable;
            }
        }

        SATSolution::Satisfiable(
            self.variables
                .into_iter()
                .map(|var| var.assignment.map(|a| a.sign).unwrap_or(false))
                .collect_vec(),
        )
    }

    fn set_variable(&mut self, literal: CNFVar, is_forced: bool) -> Option<ClauseId> {
        None
    }

    fn unit_propagation(&mut self) -> Option<ClauseId> {
        while let Some(literal) = self.unit_queue.pop_front() {
            let empty_clause = self.set_variable(literal, true);
            if empty_clause.is_some() {
                return empty_clause;
            }
        }
        None
    }

    fn backtracking(
        &mut self,
        empty_clause: ClauseId,
        learning_scheme: &impl LearningScheme,
    ) -> bool {
        let (conflict_clause, assertion_level) = learning_scheme.find_conflict_clause(empty_clause, self.branching_depth, &self.clauses, &self.variables);

        // TODO: Handle watched_literals
        self.clauses.push(conflict_clause);
        // TODO: More efficient 
        for variable in self.variables.iter_mut() {
            match variable.assignment {
                Some(assign) if assign.branching_level > assertion_level => {
                    variable.assignment = None;
                },
                _ => {},
            }
        }
        self.branching_depth = assertion_level;
        self.unit_queue.clear();

        let literal = self.clauses.last()
            .expect("There are no clauses!")
            .watched_literals
            .iter()
            .find_map(|lit| 
                if self.variables[lit.id].assignment.is_none() { Some(lit.clone()) }
                else { None }
            ).expect("Conflict clause was not unit");
        self.unit_queue.push_back(literal);
        

        self.unit_propagation()
            .map_or(false, |empty_clause| self.backtracking(empty_clause, learning_scheme))
    }
}

struct CDCLSolver<B: BranchingStrategy, L: LearningScheme, C: ClauseDeletionStrategy> {
    branching_strategy: B,
    learning_scheme: L,
    clause_deletion_strategy: C,
}

impl<B, L, C> Solver for CDCLSolver<B, L, C>
where
    B: BranchingStrategy,
    L: LearningScheme,
    C: ClauseDeletionStrategy,
{
    fn solve(&self, formula: &CNF) -> SATSolution {
        todo!()
    }
}
