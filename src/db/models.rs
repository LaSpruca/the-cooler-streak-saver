use super::schema::questions;
use crate::common::QuestionType;

#[derive(Queryable)]
pub struct Question {
    pub id: i32,
    pub question: String,
    pub answer: String,
    pub language: String,
    pub question_type: QuestionType,
}

#[derive(Insertable)]
#[table_name = "questions"]
pub struct NewPost<'a> {
    pub question: &'a str,
    pub answer: &'a str,
    pub language: &'a str,
    pub question_type: QuestionType,
}
