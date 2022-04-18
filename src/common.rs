use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::*;
use std::io;

/// Uses tokio to delay program for number of ms
#[macro_export]
macro_rules! delay {
    ($time:literal) => {
        tokio::time::sleep(std::time::Duration::from_millis($time)).await
    };
}

#[derive(Debug, Copy, Clone, AsExpression, FromSqlRow, PartialEq)]
#[sql_type = "Text"]
pub enum QuestionType {
    Translate,
    Select,
    Assist,
    TapComplete,
}

impl<DB: Backend> ToSql<Text, DB> for QuestionType
where
    str: ToSql<Text, DB>,
{
    fn to_sql<W>(&self, out: &mut Output<W, DB>) -> serialize::Result
    where
        W: io::Write,
    {
        let v = match self {
            QuestionType::Translate => "translate",
            QuestionType::Select => "select",
            QuestionType::Assist => "assist",
            QuestionType::TapComplete => "tapComplete",
        };

        v.to_sql(out)
    }
}

impl<DB: Backend> FromSql<Text, DB> for QuestionType
where
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        let v = String::from_sql(bytes)?;
        Ok(match v.as_str() {
            "translate" => QuestionType::Translate,
            "select" => QuestionType::Select,
            "assist" => QuestionType::Assist,
            "tapComplete" => QuestionType::TapComplete,
            _ => return Err("Unrecognized question type, tf are you on?".into()),
        })
    }
}
