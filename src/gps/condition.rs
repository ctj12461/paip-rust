use super::state::{StateData, StateSet};
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait Condition {
    fn check(&self, state_set: &StateSet) -> bool {
        match state_set.get(self.name()) {
            Some(state_data) => self.check_data(state_data),
            None => false,
        }
    }

    fn check_data(&self, state_data: &StateData) -> bool;

    fn name(&self) -> &str;
}

#[enum_dispatch(Condition)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConditionImpl {
    Contain,
    NotContain,
    Compare,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contain {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotContain {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Compare {
    name: String,
    operator: CompareOperator,
    value: StateData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompareOperator {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

impl Contain {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Condition for Contain {
    fn check_data(&self, _state_data: &StateData) -> bool {
        true
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl NotContain {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Condition for NotContain {
    fn check(&self, state_set: &StateSet) -> bool {
        state_set.get(self.name()).is_none()
    }

    fn check_data(&self, _state_data: &StateData) -> bool {
        false
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Compare {
    pub fn new(name: String, operator: CompareOperator, value: StateData) -> Self {
        Self {
            name,
            operator,
            value,
        }
    }
}

impl Condition for Compare {
    fn check_data(&self, state_data: &StateData) -> bool {
        match self.operator {
            CompareOperator::Equal => state_data.eq(&self.value),
            CompareOperator::NotEqual => state_data.ne(&self.value),
            CompareOperator::Greater => state_data.gt(&self.value),
            CompareOperator::GreaterEqual => state_data.ge(&self.value),
            CompareOperator::Less => state_data.lt(&self.value),
            CompareOperator::LessEqual => state_data.le(&self.value),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<&str> for CompareOperator {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "==" => Ok(Self::Equal),
            "!=" => Ok(Self::NotEqual),
            ">" => Ok(Self::Greater),
            ">=" => Ok(Self::GreaterEqual),
            "<" => Ok(Self::Less),
            "<=" => Ok(Self::LessEqual),
            _ => Err(()),
        }
    }
}
