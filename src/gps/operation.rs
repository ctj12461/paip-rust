use super::condition::ConditionImpl;
use super::state::{State, StateData, StateSet};
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::rc::Rc;

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
    name: String,
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
            let Modification { name, modification } = s;
            state_set
                .get_mut(&name)
                .and_then(|state| Some(modification(state)));
        }
    }
}

impl From<OperationInner> for Operation {
    fn from(value: OperationInner) -> Self {
        Self {
            inner: Rc::new(value),
        }
    }
}

// impl Debug for Operation {
//     fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
//         write!(f, "{}", format!("{:?}", *self.inner))
//     }
// }

impl Modification {
    pub fn new(name: String, modification: Box<dyn Fn(&mut StateData)>) -> Self {
        Self { name, modification }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Debug for Modification {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Modification {{ name: {} }}", self.name)
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
