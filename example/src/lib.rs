use es_fluent::EsFluent;

#[derive(Debug, Clone, PartialEq, Eq, Hash, EsFluent)]
pub struct NumberRangeValidation {
    min: i32,
    max: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EsFluent)]
pub struct StringLengthValidation {
    min: usize,
    max: usize,
}
