use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use proc_macro2::Span;

#[derive(Clone, Debug)]
pub struct ValidatorInfo {
    pub name: String,
    pub namespace: String,
    pub message_id: String,
    pub source: PathBuf,
    pub fields: HashSet<String>,
}

#[derive(Clone, Debug)]
pub struct DisplayInfo {
    pub expr_by_placeholder: HashMap<String, String>,
    pub source: PathBuf,
    pub write_span: Span,
}

#[derive(Clone, Debug)]
pub struct SyncTarget {
    pub validator: ValidatorInfo,
    pub display: DisplayInfo,
    pub template: Vec<TemplatePart>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemplatePart {
    Text(String),
    Placeholder(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormatChunk {
    Text(String),
    Slot(String),
}

pub type ParsedDisplay = (String, HashMap<String, String>, Span);

#[derive(Clone, Debug)]
pub struct Replacement {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
    pub type_name: String,
}
