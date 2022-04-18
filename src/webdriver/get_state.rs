use crate::common::QuestionType;
use crate::webdriver::get_state::State::{Fuckd, JustClickNext, Question, UnknownQuestionType};
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, WebDriver};
use tracing::debug;

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    StartLanguage,
    StartLesson,
    Question(QuestionType, String),
    JustClickNext,
    Fuckd,
    UnknownQuestionType(String),
}

pub async fn get_state(driver: &WebDriver) -> WebDriverResult<State> {
    // Check to see if on skill tree page
    if let Ok(elm) = driver
        .find_element(By::Css("div[data-test=\"skill-tree\"]"))
        .await
    {
        debug!("Found skill tree");
        if let Ok(_) = elm.find_element(By::Css("h2")).await {
            Ok(State::StartLanguage)
        } else {
            Ok(State::StartLesson)
        }
    }
    // Check if there is a next button
    else if let Ok(_) = driver
        .find_element(By::Css(r#"button[data-test="player-next"]"#))
        .await
    {
        debug!("Found next button");
        // Check for the challenge div
        if let Ok(question) = driver
            .find_element(By::Css(r#"div[data-test*="challenge"]"#))
            .await
        {
            debug!("Found challenge div");

            // Get the question type from the data-test attribute
            let question_type = question
                .get_attribute("data-test")
                .await?
                .unwrap()
                .strip_prefix("challenge ")
                .unwrap()
                .to_string();

            debug!("Found got question type");

            match question_type.as_str() {
                "challenge-select" => {
                    // Get the text
                    let text = driver
                        .find_element(By::Css(r#"h1[data-test="challenge-header"]"#))
                        .await?
                        .text()
                        .await?;
                    Ok(Question(QuestionType::Select, text))
                }

                "challenge-translate" => {
                    let text = driver
                        .find_element(By::Css(r#"div[data-test="challenge-header"]"#))
                        .await?
                        .text()
                        .await?;
                    Ok(Question(QuestionType::Translate, text))
                }

                _ => Ok(UnknownQuestionType(question_type)),
            }
        } else {
            Ok(JustClickNext)
        }
    } else {
        Ok(Fuckd)
    }
}
