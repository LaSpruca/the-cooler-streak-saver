use super::schema::questions;
use crate::common::QuestionType;

#[derive(Queryable, Debug, Clone)]
pub struct Question {
    pub id: i32,
    pub language: String,
    pub question: String,
    pub answer: String,
    pub question_type: QuestionType,
}

#[derive(Insertable, Clone, Debug)]
#[table_name = "questions"]
pub struct NewQuestion {
    pub language: String,
    pub question: String,
    pub answer: String,
    pub question_type: QuestionType,
}
