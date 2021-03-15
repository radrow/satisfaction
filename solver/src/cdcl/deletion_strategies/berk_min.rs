use std::collections::VecDeque;

use itertools::Itertools;
use tinyset::SetUsize;

use super::{
    ClauseDeletionStrategy,
    ClauseDeletionStrategyFactory,
    super::{
        clause::{ClauseId, Clauses},
        update::Update,
        util::HashMap,
        variable::Variables,
    },
};

pub struct BerkMinInstance {
    activity: HashMap<ClauseId, usize>,
    insertion_order: VecDeque<ClauseId>,
    threshold: usize,
}

impl Update for BerkMinInstance {
    fn on_learn(&mut self, learned_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        self.insertion_order.push_back(learned_clause);
        self.activity.insert(learned_clause, 0);
    }

    fn on_conflict(&mut self, empty_clause: ClauseId, _clauses: &Clauses, _variables: &Variables) {
        *self.activity.get_mut(&empty_clause)
            .expect("Empty clause was not registered!") +=1;
    }
}

impl ClauseDeletionStrategy for BerkMinInstance {
    fn delete_clause(&mut self, clauses: &Clauses, variables: &Variables) -> Vec<ClauseId> {
        let pct = clauses.len() / 16;
        let unassigned = self.insertion_order.iter()
            .cloned()
            .filter(|c| {
                clauses[*c].watched_literals.iter().all(|i| { variables[clauses[*c].literals[*i].id].assignment.is_none() })
            });

        let young = unassigned.clone().take(pct);
        let old = unassigned.skip(pct);

        let to_be_deleted: SetUsize= young.filter(|id| 42 < clauses[*id].literals.len() && self.activity[id] < 7)
            .chain(old.filter(|id| 8 < clauses[*id].literals.len() && self.activity[id] < self.threshold)).collect();

        self.activity.retain(|k,_| !to_be_deleted.contains(*k));
        self.insertion_order.retain(|k| !to_be_deleted.contains(*k));

        self.threshold += 1;

        to_be_deleted.into_iter().collect_vec()
    }
}

pub struct BerkMin(usize);

impl ClauseDeletionStrategyFactory for BerkMin {
    fn create(&self, _clauses: &Clauses, _variables: &Variables) -> Box<dyn ClauseDeletionStrategy> {
        Box::new(BerkMinInstance {
            activity: HashMap::default(),
            insertion_order: VecDeque::new(),
            threshold: self.0,
        })
    }
}

impl Default for BerkMin {
    fn default() -> Self {
        BerkMin(60)
    }
}
