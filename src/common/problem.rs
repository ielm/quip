use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{QUESTION_QUERY_OPERATION, QUESTION_QUERY_STRING};

use std::fmt::{Display, Error, Formatter};

#[derive(Serialize, Deserialize)]
pub struct Problem {
    pub title: String,
    pub title_slug: String,
    pub content: String,
    #[serde(rename = "codeDefinition")]
    pub code_definition: Vec<CodeDefinition>,
    #[serde(rename = "sampleTestCase")]
    pub sample_test_case: String,
    pub difficulty: String,
    pub question_id: u32,
    pub return_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct CodeDefinition {
    pub value: String,
    pub text: String,
    #[serde(rename = "defaultCode")]
    pub default_code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Query {
    #[serde(rename = "operationName")]
    operation_name: String,
    variables: serde_json::Value,
    query: String,
}

impl Query {
    pub fn question_query(title_slug: &str) -> Query {
        Query {
            operation_name: QUESTION_QUERY_OPERATION.to_owned(),
            variables: json!({ "titleSlug": title_slug }),
            query: QUESTION_QUERY_STRING.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawProblem {
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub question: Question,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    pub content: String,
    pub stats: String,
    #[serde(rename = "codeDefinition")]
    pub code_definition: String,
    #[serde(rename = "sampleTestCase")]
    pub sample_test_case: String,
    #[serde(rename = "metaData")]
    pub meta_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProblems {
    pub user_name: String,
    pub num_solved: u32,
    pub num_total: u32,
    pub ac_easy: u32,
    pub ac_medium: u32,
    pub ac_hard: u32,
    pub stat_status_pairs: Vec<StatWithStatus>,
    pub frequency_high: f32,
    pub frequency_mid: f32,
    pub category_slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatWithStatus {
    pub stat: Stat,
    pub status: Option<String>,
    pub difficulty: Difficulty,
    pub paid_only: bool,
    pub is_favor: bool,
    pub frequency: f32,
    pub progress: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stat {
    pub question_id: u32,
    #[serde(rename = "question__article__slug")]
    pub question_article_slug: Option<String>,
    #[serde(rename = "question__title")]
    pub question_title: Option<String>,
    #[serde(rename = "question__title_slug")]
    pub question_title_slug: Option<String>,
    #[serde(rename = "question__hide")]
    pub question_hide: bool,
    pub total_acs: u32,
    pub total_submitted: u32,
    pub frontend_question_id: u32,
    pub is_new_question: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Difficulty {
    pub level: u32,
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self.level {
            1 => f.write_str("Easy"),
            2 => f.write_str("Medium"),
            3 => f.write_str("Hard"),
            _ => f.write_str("Unknown"),
        }
    }
}
