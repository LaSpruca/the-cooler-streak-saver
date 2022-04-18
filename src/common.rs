use diesel::sql_types::Text;

#[derive(AsExpression, Debug, Clone, PartialEq)]
#[sql_type = "Text"]
pub enum QuestionType {
    Translate,
    Select,
}
