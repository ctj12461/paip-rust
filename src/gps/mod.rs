pub mod condition;
pub mod operation;
pub mod state;

use condition::{Condition, ConditionImpl};
use operation::Operation;
use state::StateSet;

use self::condition::ConditionSet;

pub struct GeneralProblemSolver {
    operations: Vec<Operation>,
    goals: Vec<ConditionImpl>,
    states: StateSet,
}

impl GeneralProblemSolver {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            goals: Vec::new(),
            states: StateSet::new(),
        }
    }

    pub fn set_operations(&mut self, operation: Vec<Operation>) -> &mut Self {
        self.operations = operation;
        self
    }

    pub fn set_goals(&mut self, goals: Vec<ConditionImpl>) -> &mut Self {
        self.goals = goals;
        self
    }

    pub fn set_states(&mut self, states: StateSet) -> &mut Self {
        self.states = states;
        self
    }

    /// Solve the given problem and return the solution.
    pub fn solve(&self) -> Option<Vec<Operation>> {
        let mut goal_stack = Vec::new();
        let mut protected_goals = ConditionSet::new();

        self.solve_all(
            &self.goals,
            &self.states,
            &mut goal_stack,
            &mut protected_goals,
        )
        .map(|(_, operations)| operations)
    }

    /// Achieve a set of goals and return operations required and states
    /// after this procdure.
    fn solve_all(
        &self,
        goals: &Vec<ConditionImpl>,
        current_states: &StateSet,
        goal_stack: &mut Vec<ConditionImpl>,
        protected_goals: &mut ConditionSet,
    ) -> Option<(StateSet, Vec<Operation>)> {
        if current_states.has_reached(&goals) {
            return Some((current_states.clone(), Vec::new()));
        }

        let mut new_states = current_states.clone();
        let mut unachieved_goals = Vec::new();

        for goal in goals {
            if goal.check(current_states) {
                // Already achieved goals shouldn't be destoryed by other operations.
                protected_goals.insert(goal.state_name(), goal.clone());
            } else {
                unachieved_goals.push(goal.clone());
            }
        }

        let mut operations = Vec::new();

        // Achieve each unachieved goal.
        for goal in &unachieved_goals {
            let (next_states, mut next_operations) =
                self.solve_one(goal, &new_states, goal_stack, protected_goals)?;
            protected_goals.insert(goal.state_name(), goal.clone());
            operations.append(&mut next_operations);
            new_states = next_states;
        }

        // Ensure all goals have been achieved.
        if goals.iter().all(|condition| condition.check(&new_states)) {
            goals.iter().for_each(|goal| {
                protected_goals.remove(goal.state_name(), goal);
            });
            Some((new_states, operations))
        } else {
            None
        }
    }

    /// Achieve one individual goal and return operations required and states
    /// after this procdure.
    fn solve_one(
        &self,
        goal: &ConditionImpl,
        current_states: &StateSet,
        goal_stack: &mut Vec<ConditionImpl>,
        protected_goals: &mut ConditionSet,
    ) -> Option<(StateSet, Vec<Operation>)> {
        if goal.check(current_states) {
            return Some((current_states.clone(), Vec::new()));
        }

        if goal_stack.contains(&goal) {
            return None;
        }

        let valid_operations = self.find_valid_operations(goal, current_states, protected_goals);
        goal_stack.push(goal.clone());

        for valid_operation in valid_operations.iter() {
            let res = self.apply_operation(
                valid_operation.clone(),
                current_states,
                goal_stack,
                protected_goals,
            );

            if res.is_some() {
                goal_stack.pop();
                return res;
            }
        }

        goal_stack.pop();
        None
    }

    /// Find out all operations capable of achieving the given goal.
    fn find_valid_operations(
        &self,
        goal: &ConditionImpl,
        current_states: &StateSet,
        protected_goals: &ConditionSet,
    ) -> Vec<Operation> {
        match goal {
            ConditionImpl::Contain(_) => self
                .operations
                .iter()
                .filter(|operation| {
                    // Check if this operation will add the needed state.
                    operation
                        .add_states()
                        .iter()
                        .find(|state| state.name() == goal.name())
                        .is_some()
                })
                // Ensure that protects goals will be conserved.
                .filter(|operation| !operation.has_affect(current_states, protected_goals))
                .cloned()
                .collect(),
            ConditionImpl::NotContain(_) => self
                .operations
                .iter()
                .filter(|operation| {
                    // Check if this operation will remove the target state.
                    operation
                        .remove_states()
                        .iter()
                        .find(|state_name| state_name.as_str() == goal.name())
                        .is_some()
                })
                .filter(|operation| !operation.has_affect(current_states, protected_goals))
                .cloned()
                .collect(),
            ConditionImpl::Compare(_) => self
                .operations
                .iter()
                .filter(|operation| {
                    // Check if this operation will modify the target state.
                    operation
                        .modification_states()
                        .iter()
                        .find(|modification| modification.target_name() == goal.state_name())
                        .is_some()
                })
                .filter(|operation| !operation.has_affect(current_states, protected_goals))
                .cloned()
                .collect(),
        }
    }

    fn apply_operation(
        &self,
        target_operation: Operation,
        current_states: &StateSet,
        goal_stack: &mut Vec<ConditionImpl>,
        protected_goals: &mut ConditionSet,
    ) -> Option<(StateSet, Vec<Operation>)> {
        // Achieve all the target operation's prerequisites first.
        match self.solve_all(
            target_operation.prerequisites(),
            current_states,
            goal_stack,
            protected_goals,
        ) {
            Some((mut next_states, mut operations)) => {
                target_operation.apply(&mut next_states);
                operations.push(target_operation);
                Some((next_states, operations))
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use condition::Compare;
    use condition::Contain;
    use operation::Modification;
    use operation::OperationBuilder;
    use state::{State, StateData};

    #[test]
    fn it_should_return_valid_operations_that_add_the_state() {
        let mut gps = GeneralProblemSolver::new();
        let goal: ConditionImpl = Contain::new("state".to_owned()).into();

        gps.set_operations(vec![
            OperationBuilder::new("add-state".to_owned())
                .insert_add_state(State::new_symbol("state".to_owned()))
                .build(),
            OperationBuilder::new("add-state-with-prerequisite".to_owned())
                .insert_prerequisite(Contain::new("prerequisite".to_owned()).into())
                .insert_add_state(State::new_symbol("state".to_owned()))
                .build(),
            OperationBuilder::new("add-another-state".to_owned())
                .insert_add_state(State::new_symbol("another-state".to_owned()))
                .build(),
        ]);

        let operations =
            gps.find_valid_operations(&goal, &StateSet::new(), &mut ConditionSet::new());
        assert!(operations
            .iter()
            .find(|operation| operation.name() == "add-state")
            .is_some());
        assert!(operations
            .iter()
            .find(|operation| operation.name() == "add-state-with-prerequisite")
            .is_some());
        assert!(operations
            .iter()
            .find(|operation| operation.name() == "add-another-state")
            .is_none());
    }

    #[test]
    fn it_should_return_the_valid_operations_that_satisify_the_conditions() {
        let mut gps = GeneralProblemSolver::new();

        gps.set_operations(vec![
            OperationBuilder::new("add-10".to_owned())
                .insert_modify_state(Modification::new(
                    "value".to_owned(),
                    Box::new(|data| {
                        let new_data = match data {
                            StateData::Symbol => StateData::Integer(0),
                            StateData::Integer(x) => StateData::Integer(*x + 10),
                        };
                        *data = new_data;
                    }),
                ))
                .build(),
            OperationBuilder::new("add-50".to_owned())
                .insert_modify_state(Modification::new(
                    "value".to_owned(),
                    Box::new(|data| {
                        let new_data = match data {
                            StateData::Symbol => StateData::Integer(0),
                            StateData::Integer(x) => StateData::Integer(*x + 50),
                        };
                        *data = new_data;
                    }),
                ))
                .build(),
        ]);

        let goal: ConditionImpl = Compare::new(
            "less-than-20".to_owned(),
            "value".to_owned(),
            condition::CompareOperator::Less,
            StateData::Integer(20),
        )
        .into();

        let current_states = {
            let mut state_set = StateSet::new();
            state_set.insert(State::new_integer("value".to_owned(), 0));
            state_set
        };

        let mut condition_set = ConditionSet::new();
        condition_set.insert("value", goal.clone());

        let operations = gps.find_valid_operations(&goal, &current_states, &condition_set);
        assert_eq!(operations.first().unwrap().name(), "add-10");
        assert_eq!(operations.len(), 1);
    }

    #[test]
    fn it_should_achieve_the_goal() {
        // Test case from paip-lisp
        let mut gps = GeneralProblemSolver::new();

        gps.set_operations(test_operations())
            .set_goals(vec![Contain::new("son-at-school".to_owned()).into()])
            .set_states({
                let mut states = StateSet::new();
                states.insert(State::new_symbol("son-at-home".to_owned()));
                states.insert(State::new_symbol("car-needs-battery".to_owned()));
                states.insert(State::new_symbol("have-money".to_owned()));
                states.insert(State::new_symbol("have-phone-book".to_owned()));
                states
            });

        match gps.solve() {
            Some(operations) => {
                let mut iter = operations.iter();
                assert_eq!(iter.next().unwrap().name(), "look-up-number");
                assert_eq!(iter.next().unwrap().name(), "telephone-shop");
                assert_eq!(iter.next().unwrap().name(), "tell-shop-problem");
                assert_eq!(iter.next().unwrap().name(), "give-shop-money");
                assert_eq!(iter.next().unwrap().name(), "shop-installs-battery");
                assert_eq!(iter.next().unwrap().name(), "drive-son-to-school");
                assert!(iter.next().is_none());
            }
            None => unreachable!(),
        }
    }

    #[test]
    fn is_should_return_none_when_solving_recursive_subgoals() {
        let mut gps = GeneralProblemSolver::new();

        gps.set_operations({
            let mut operations = test_operations();
            operations.push(
                OperationBuilder::new("ask-phone-number".to_owned())
                    .insert_prerequisite(
                        Contain::new("in-communication-with-shop".to_owned()).into(),
                    )
                    .insert_add_state(State::new_symbol("know-phone-number".to_owned()))
                    .build(),
            );
            operations
        })
        .set_goals(vec![Contain::new("son-at-school".to_owned()).into()])
        .set_states({
            let mut states = StateSet::new();
            states.insert(State::new_symbol("son-at-home".to_owned()));
            states.insert(State::new_symbol("car-needs-battery".to_owned()));
            states.insert(State::new_symbol("have-money".to_owned()));
            states
        });

        let res = gps.solve();
        assert!(res.is_none());
    }

    #[test]
    fn it_should_try_another_operation_to_solve_goals_after_first_operation_failed() {
        let mut gps = GeneralProblemSolver::new();

        gps.set_operations({
            let mut operations = test_operations();
            operations.push(
                OperationBuilder::new("taxi-son-to-school".to_owned())
                    .insert_prerequisite(Contain::new("son-at-home".to_owned()).into())
                    .insert_prerequisite(Contain::new("have-money".to_owned()).into())
                    .insert_add_state(State::new_symbol("son-at-school".to_owned()))
                    .insert_remove_state("son-at-home".to_owned())
                    .insert_remove_state("have-money".to_owned())
                    .build(),
            );
            operations
        })
        .set_goals(vec![
            Contain::new("son-at-school".to_owned()).into(),
            Contain::new("have-money".to_owned()).into(),
        ])
        .set_states({
            let mut states = StateSet::new();
            states.insert(State::new_symbol("son-at-home".to_owned()));
            states.insert(State::new_symbol("have-money".to_owned()));
            states.insert(State::new_symbol("car-works".to_owned()));
            states
        });

        match gps.solve() {
            Some(operations) => {
                let mut iter = operations.iter();
                assert_eq!(iter.next().unwrap().name(), "drive-son-to-school");
                assert!(iter.next().is_none());
            }
            None => unreachable!(),
        }
    }

    fn test_operations() -> Vec<Operation> {
        vec![
            OperationBuilder::new("drive-son-to-school".to_owned())
                .insert_prerequisite(Contain::new("son-at-home".to_owned()).into())
                .insert_prerequisite(Contain::new("car-works".to_owned()).into())
                .insert_add_state(State::new_symbol("son-at-school".to_owned()))
                .insert_remove_state("son-at-home".to_owned())
                .build(),
            OperationBuilder::new("shop-installs-battery".to_owned())
                .insert_prerequisite(Contain::new("car-needs-battery".to_owned()).into())
                .insert_prerequisite(Contain::new("shop-knows-problem".to_owned()).into())
                .insert_prerequisite(Contain::new("shop-has-money".to_owned()).into())
                .insert_add_state(State::new_symbol("car-works".to_owned()))
                .build(),
            OperationBuilder::new("tell-shop-problem".to_owned())
                .insert_prerequisite(Contain::new("in-communication-with-shop".to_owned()).into())
                .insert_add_state(State::new_symbol("shop-knows-problem".to_owned()))
                .build(),
            OperationBuilder::new("telephone-shop".to_owned())
                .insert_prerequisite(Contain::new("know-phone-number".to_owned()).into())
                .insert_add_state(State::new_symbol("in-communication-with-shop".to_owned()))
                .build(),
            OperationBuilder::new("look-up-number".to_owned())
                .insert_prerequisite(Contain::new("have-phone-book".to_owned()).into())
                .insert_add_state(State::new_symbol("know-phone-number".to_owned()))
                .build(),
            OperationBuilder::new("give-shop-money".to_owned())
                .insert_prerequisite(Contain::new("have-money".to_owned()).into())
                .insert_add_state(State::new_symbol("shop-has-money".to_owned()))
                .insert_remove_state("have-money".to_owned())
                .build(),
        ]
    }
}
