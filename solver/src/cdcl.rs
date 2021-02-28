use std::collections::{HashSet, VecDeque};
use crate::{CNF, CNFClause, CNFVar, SATSolution, Solver};


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
    fn pick_literal(&self, clause: &Clauses, variables: &Variables) -> Option<CNFVar>;
}

trait LearningScheme {
    fn find_conflict_clause(&self, clauses: &Clauses, variables: &Variables) -> Option<Clause>;
}

trait ClauseDeletionStrategy {
    fn delete_clause(&self, clauses: &mut Clauses, variables: &mut Variables);
}

struct ExecutionState{
    clauses: Clauses,
    variables: Variables,
    branching_depth: usize,
    unit_queue: VecDeque<CNFVar>,
}

impl ExecutionState {
    fn cdcl(&mut self, branching_strategy: &impl BranchingStrategy, learning_scheme: &impl LearningScheme, clause_deletion_strategy: &impl ClauseDeletionStrategy) -> SATSolution {
        todo!()
    }
}


struct CDCLSolver<B: BranchingStrategy, L: LearningScheme, C: ClauseDeletionStrategy> {
    branching_strategy: B,
    learning_scheme: L,
    clause_deletion_strategy: C,
}

impl<B,L,C> Solver for CDCLSolver<B, L, C>
where B: BranchingStrategy,
      L: LearningScheme,
      C: ClauseDeletionStrategy {
    fn solve(&self, formula: &CNF) -> SATSolution {
       todo!() 
    }
}
