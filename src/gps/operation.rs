use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::rc::Rc;

use super::condition::{Condition, ConditionImpl, ConditionSet};
use super::state::{State, StateData, StateSet};

#[derive(Debug, Clone)]
pub struct Operation {
    inner: Rc<OperationInner>,
}

#[derive(Debug)]
struct OperationInner {
    name: String,
    prerequisites: Vec<ConditionImpl>,
    add_states: Vec<State>,
    remove_states: Vec<String>,
    modify_states: Vec<Modification>,
}

pub struct Modification {
    target_name: String,
    modification: Box<dyn Fn(&mut StateData)>,
}

pub struct OperationBuilder {
    object: OperationInner,
}

impl Operation {
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    pub fn prerequisites(&self) -> &Vec<ConditionImpl> {
        &self.inner.prerequisites
    }

    pub fn add_states(&self) -> &Vec<State> {
        &self.inner.add_states
    }

    pub fn remove_states(&self) -> &Vec<String> {
        &self.inner.remove_states
    }

    pub fn modification_states(&self) -> &Vec<Modification> {
        &self.inner.modify_states
    }

    pub fn apply(&self, state_set: &mut StateSet) {
        for s in &self.inner.add_states {
            state_set.insert(s.clone());
        }

        for s in &self.inner.remove_states {
            state_set.remove(s);
        }

        for s in &self.inner.modify_states {
            let Modification {
                target_name: name,
                modification,
            } = s;
            state_set
                .get_mut(&name)
                .and_then(|state| Some(modification(state)));
        }
    }

    /// Test if applying this operation will have impact on the given
    /// goals.
    pub fn has_affect(&self, current_states: &StateSet, goals: &ConditionSet) -> bool {
        for state in self.add_states() {
            let Some(conds) = goals.get(state.name()) else {
                continue;
            };

            if conds.iter().any(|cond| {
                if cond.state_name() != state.name() {
                    return false;
                }
                matches!(cond, ConditionImpl::NotContain(_))
            }) {
                return true;
            }
        }

        for state_name in self.remove_states() {
            let Some(conds) = goals.get(state_name) else {
                continue;
            };

            if conds.iter().any(|cond| {
                if cond.state_name() != state_name {
                    return false;
                }
                if matches!(cond, ConditionImpl::Contain(_)) {
                    true
                } else if matches!(cond, ConditionImpl::Compare(_)) {
                    true
                } else {
                    false
                }
            }) {
                return true;
            }
        }

        for modification in self.modification_states() {
            let Some(conds) = goals.get(modification.target_name()) else {
                continue;
            };

            if conds.iter().any(|cond| {
                let Some(state_data) = current_states.get(modification.target_name()) else {
                    return false;
                };

                let mut tmp = state_data.clone();
                (modification.modification)(&mut tmp);
                !cond.check_data(&tmp)
            }) {
                return true;
            }
        }

        return false;
    }
}

impl From<OperationInner> for Operation {
    fn from(value: OperationInner) -> Self {
        Self {
            inner: Rc::new(value),
        }
    }
}

impl Modification {
    pub fn new(target_name: String, modification: Box<dyn Fn(&mut StateData)>) -> Self {
        Self {
            target_name,
            modification,
        }
    }

    pub fn target_name(&self) -> &str {
        &self.target_name
    }
}

impl Debug for Modification {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Modification {{ name: {} }}", self.target_name)
    }
}

impl OperationBuilder {
    pub fn new(name: String) -> Self {
        Self {
            object: OperationInner {
                name,
                prerequisites: Vec::new(),
                add_states: Vec::new(),
                remove_states: Vec::new(),
                modify_states: Vec::new(),
            },
        }
    }

    pub fn insert_prerequisite(mut self, condition: ConditionImpl) -> Self {
        self.object.prerequisites.push(condition);
        self
    }

    pub fn insert_add_state(mut self, add_state: State) -> Self {
        self.object.add_states.push(add_state);
        self
    }

    pub fn insert_remove_state(mut self, remove_state: String) -> Self {
        self.object.remove_states.push(remove_state);
        self
    }

    pub fn insert_modify_state(mut self, modify_state: Modification) -> Self {
        self.object.modify_states.push(modify_state);
        self
    }

    pub fn build(self) -> Operation {
        self.object.into()
    }
}
