use std::str::FromStr;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct QuestionId(String);

impl FromStr for QuestionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

impl Question {
    pub fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
        Self { id, title, content, tags }
    }
}
