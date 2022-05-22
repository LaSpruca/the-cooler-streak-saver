#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate core;

use crate::common::QuestionType;
use crate::db::models::{NewQuestion, Question};
use crate::db::{create_connection, run_migrations, DbConnection};
use crate::webdriver::{
    answer_multi_question, answer_question, discard_question, get_state, next, skip_question,
    start_language, State, WebdriverSender,
};
use diesel::prelude::*;
use dotenv::dotenv;
use std::collections::HashMap;
use std::process::exit;
use tokio::signal::ctrl_c;
use tracing::{error, info};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use webdriver::start_lesson;

mod common;
mod db;
mod webdriver;

#[tokio::main]
async fn main() {
    // Load .env file
    match dotenv() {
        Ok(_) => {}
        Err(_) => {}
    };

    // Setup logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // Connect to the database
    let db = match create_connection() {
        None => exit(1),
        Some(db) => db,
    };

    match run_migrations(&db) {
        Ok(_) => {}
        Err(ex) => {
            error!("Error running migrations {ex:?}");
            panic!("Could not run migrations")
        }
    };

    // Open a chrome window with thirtyfour
    let tx = match webdriver::open_browser().await {
        Ok(d) => d,
        Err(e) => {
            error!("Could not load webdriver {e}");
            exit(1);
        }
    };

    // Setup control_c handler to quit the chrome instance
    // let yeet = tx.clone();
    // tokio::spawn(async move {
    //     ctrl_c().await.unwrap();
    //     info!("Control c'd");
    //     webdriver::quit(yeet).await;
    //     exit(0);
    // });

    // Run the main application with panic capture so that the chrome window can be closed
    match tokio::spawn(run(tx.clone(), db)).await {
        Err(e) => {
            if e.is_panic() {
                error!("Application panicked");
            }
        }
        _ => {}
    };

    // Yeet chrome, because fuck you
    // webdriver::quit(tx).await;
}

async fn run(tx: WebdriverSender, db_conn: DbConnection) {
    match webdriver::sign_in(&tx).await {
        Ok(_) => {}
        Err(err) => {
            error!("{}", err);
            panic!("Error signing in");
        }
    };

    loop {
        let state = match get_state(&tx).await {
            Ok(d) => d,
            Err(ex) => {
                error!("{ex}");
                loop {}
            }
        };

        match state {
            State::StartLanguage => {
                start_language(&tx).await.unwrap();
            }
            State::StartLesson => {
                start_lesson(&tx).await.unwrap();
            }
            State::Question(qtype, lang, qu) => {
                info!("Question: Type = {qtype:?}, language = {lang}, question = {qu}");

                let answers: Vec<Question> = {
                    use db::schema::questions::dsl::*;

                    questions
                        .filter(question_type.eq(qtype.clone()))
                        .filter(language.eq(lang.clone()))
                        .filter(question.eq(qu.clone()))
                        .load::<Question>(&db_conn)
                        .unwrap()
                };

                if answers.is_empty() {
                    let ans = skip_question(&tx, qtype).await.unwrap();
                    {
                        use db::schema::questions;
                        let qu = NewQuestion {
                            answer: ans,
                            question: qu,
                            language: lang,
                            question_type: qtype,
                        };
                        diesel::insert_into(questions::table)
                            .values(qu.clone())
                            .execute(&db_conn)
                            .unwrap();
                        info!("Registered {:#?}", qu)
                    }
                } else {
                    let ans = answers.get(0).unwrap();

                    if let Some(updated) =
                        answer_question(&tx, ans.answer.clone(), ans.question_type.clone())
                            .await
                            .unwrap()
                    {
                        use db::schema::questions::dsl::{answer, questions};
                        diesel::update(questions.find(ans.id))
                            .set(answer.eq(updated.clone()))
                            .execute(&db_conn)
                            .unwrap();
                        info!(
                            "Updating answer to question `{}` (lang: {}, type: {:?}), to {updated}",
                            ans.question, ans.language, ans.question_type
                        );
                    }
                }
            }
            State::JustClickNext => {
                next(&tx).await.unwrap();
            }
            State::Fuckd => {
                error!("Unable to determine application state");
                continue;
            }
            State::UnknownQuestionType(kind) => {
                info!("Unknown question type: {kind}");
            }
            State::IgnoreQuestion => {
                discard_question(&tx).await.unwrap();
            }
            State::MatchQuestion(questions, lang) => {
                let answers = questions
                    .into_iter()
                    .map(|qu| {
                        (qu.clone(), {
                            {
                                use db::schema::questions::dsl::*;

                                questions
                                    .filter(question_type.eq(QuestionType::MatchPairs))
                                    .filter(language.eq(lang.clone()))
                                    .filter(question.eq(qu))
                                    .load::<Question>(&db_conn)
                                    .unwrap()
                                    .iter()
                                    .next()
                                    .map(|f| f.to_owned())
                            }
                        })
                    })
                    .collect::<HashMap<String, Option<Question>>>();

                let correct = answer_multi_question(
                    &tx,
                    answers
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (k, v.map(|v| v.answer.to_string())))
                        .collect(),
                )
                .await
                .unwrap();

                for (qu, updated) in correct.iter() {
                    if let Some(ans) = answers.get(qu).unwrap() {
                        use db::schema::questions::dsl::{answer, questions};
                        diesel::update(questions.find(ans.id))
                            .set(answer.eq(answer.clone()))
                            .execute(&db_conn)
                            .unwrap();
                        info!(
                            "Updating answer to question `{qu}` (lang: {lang}, type: {:?}), to {updated}",
                            QuestionType::MatchPairs
                        );
                    }
                }
            }
        }
    }
}
