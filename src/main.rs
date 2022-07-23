#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate core;

use crate::{
    common::QuestionType,
    db::{
        create_connection,
        models::{NewQuestion, Question},
        run_migrations, DbConnection,
    },
    webdriver::*,
};
use diesel::prelude::*;
use dotenv::dotenv;
use std::{collections::HashMap, process::exit};
use thirtyfour_sync::{prelude::WebDriverResult, WebDriver};
use tracing::{error, info};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

mod common;
mod db;
mod webdriver;

fn main() {
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
    let driver = match webdriver::open_browser() {
        Ok(d) => d,
        Err(e) => {
            error!("Could not load webdriver {e}");
            exit(1);
        }
    };

    if let Err(ex) = run(driver, db) {
        error!("{ex}");
        loop {}
    }

    // Run the main application with panic capture so that the chrome window can be closed

    // Yeet chrome, because fuck you
    // webdriver::quit(driver);
}

fn run(driver: WebDriver, db_conn: DbConnection) -> WebDriverResult<()> {
    match webdriver::browser_login(&driver) {
        Ok(_) => {}
        Err(err) => {
            error!("{}", err);
            panic!("Error signing in");
        }
    };

    loop {
        let state = match get_state(&driver) {
            Ok(d) => d,
            Err(ex) => {
                error!("{ex}");
                loop {}
            }
        };

        match state {
            State::StartLanguage => {
                start_intro(&driver)?;
            }
            State::StartLesson => {
                next_skill_tree_item(&driver)?;
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
                    let ans = if qtype.check_underline() {
                        skip_underline(&driver)?
                    } else {
                        skip(&driver)?
                    };

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

                    assert!(ans.answer.len() != 0);

                    if let Some(updated) =
                        answer_question(&driver, ans.question_type.clone(), ans.answer.clone())?
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
                click_next(&driver)?;
            }
            State::Fuckd => {
                error!("Unable to determine application state");
                continue;
            }
            State::UnknownQuestionType(kind) => {
                info!("Unknown question type: {kind}");
                delay!(500)
            }
            State::IgnoreQuestion => {
                ignore_question(&driver)?;
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

                let correct = answer_match(
                    &driver,
                    &answers
                        .clone()
                        .into_iter()
                        .map(|(k, v)| (k, v.map(|v| v.answer.to_string())))
                        .collect(),
                )?;

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
                    } else {
                        use db::schema::questions;
                        let qu = NewQuestion {
                            answer: updated.clone(),
                            question: qu.clone(),
                            language: lang.clone(),
                            question_type: QuestionType::MatchPairs,
                        };
                        diesel::insert_into(questions::table)
                            .values(qu.clone())
                            .execute(&db_conn)
                            .unwrap();
                        info!("Registered {:#?}", qu)
                    }
                }
            }
            State::PlusScreen => {
                click_on(&driver, "plus-no-thanks")?;
            }
            State::Legendary => click_on(&driver, "maybe-later")?,
            State::HereIsATip => {
                here_is_tip(&driver)?;
            }
            State::Loading => {
                delay!(100);
            }
        }
    }
}
