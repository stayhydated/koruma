use es_fluent::EsFluent;

#[derive(Debug, Clone, PartialEq, Eq, Hash, EsFluent)]
pub enum NumberValidation {
    Range { min: i32, max: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, EsFluent)]
pub enum StringValidation {
    Length { min: usize, max: usize },
}
