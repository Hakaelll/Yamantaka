use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BookStudy {
    pub revision: u64,
    pub sections: Vec<Section>,
    pub cards: Vec<Card>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub text: String,
    pub source: Option<Source>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub annotation: String,
    pub page: u32,
    pub point: [f64; 2],
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Card {
    pub question: String,
    pub answer: String,
    pub source: Option<Source>,
    pub due: Option<i64>,
    pub interval: usize,
}

pub enum Operation {
    Load,
    Save(BookStudy),
    Export(PathBuf),
}

pub struct Request {
    pub document: String,
    pub operation: Operation,
}

pub struct Reply {
    pub document: String,
    pub result: Result<BookStudy>,
    pub exported: bool,
}
