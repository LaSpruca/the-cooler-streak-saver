use crate::common::QuestionType;
use crate::delay;
use crate::webdriver::get_state::State::{Fuckd, JustClickNext, Question, UnknownQuestionType};
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, WebDriver};
use tracing::{debug, info};

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    StartLanguage,
    StartLesson,
    /// - `0`: The kind of question
    /// - `1`: The language
    /// - `2`: The question itself
    Question(QuestionType, String, String),
    /// This fucker is special so it gets its own enum variant
    MatchQuestion(Vec<String>, String),
    JustClickNext,
    Fuckd,
    UnknownQuestionType(String),
    IgnoreQuestion,
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
            debug!("Starting Lesson!");
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

            let language = driver
                .current_url()
                .await?
                .strip_prefix("https://www.duolingo.com/skill/")
                .unwrap()
                .split_once("/")
                .unwrap()
                .0
                .to_string();

            match question_type.as_str() {
                "challenge-select" => {
                    // Get the text
                    let text = driver
                        .find_element(By::Css(r#"h1[data-test="challenge-header"]"#))
                        .await?
                        .text()
                        .await?;
                    Ok(Question(QuestionType::Select, language, text))
                }

                "challenge-translate" => {
                    let text = driver
                        .find_element(By::Css(r#"[data-test="challenge challenge-translate"] > div > div > div > div> div> div > div"#))
                        .await?
                        .text()
                        .await?;
                    Ok(Question(QuestionType::Translate, language, text))
                }

                "challenge-assist" => {
                    let text = driver
                        .find_element(By::Css(r#"h1[data-test="challenge-header"]"#))
                        .await?
                        .text()
                        .await?;
                    Ok(Question(QuestionType::Assist, language, text))
                }

                "challenge-tapComplete" => {
                    let mut text = String::new();
                    for element in driver.find_elements(By::Css(r#"[data-test="challenge challenge-tapComplete"] > div > div > div > span"#)).await? {
                        let elm_text = element.text().await?;
                        text += elm_text.as_str();
                    }

                    Ok(Question(
                        QuestionType::TapComplete,
                        language,
                        text.trim_end().to_string(),
                    ))
                }

                "challenge-completeReverseTranslation" => {
                    // We can turn this into a translate using the make harder button
                    driver.find_element(By::Css(r#"[data-test="player-toggle-keyboard"]"#)).await?.click().await?;

                    while driver.find_element(By::Css(r#"[data-test="challenge-translate-input"]"#)).await.is_err() {
                        delay!(100)
                    }

                    let text = driver
                        .find_element(By::Css(r#"[data-test="challenge challenge-completeReverseTranslation"] > div > div > div > div> div> div > div"#))
                        .await?
                        .text()
                        .await?;
                    Ok(Question(QuestionType::Translate, language, text))

                }

                "challenge-match" => {
                    let rt = tokio::runtime::Handle::current();
                    let mut questions = vec![];

                    // Get all of the listed questions
                    for element in driver
                        .find_elements(
                            By::Css(r#"[data-test="challenge challenge-match"] > div > div > div > div > div:nth-child(1) > div > button"#)
                        )
                        .await?
                        .into_iter() {

                        let text = element.text().await?;
                        let span_text = element
                            .find_element(By::Tag("span"))
                            .await?
                            .text()
                            .await?;

                        questions.push(text
                            .strip_prefix(&span_text)
                            .unwrap()
                            .strip_prefix("\n")
                            .unwrap()
                            .to_string());
                    }

                    info!("Questions: {questions:?}");

                    Ok(State::MatchQuestion(questions, language))
                }

                "challenge-listenTap" => return Ok(State::IgnoreQuestion),

                _ => Ok(UnknownQuestionType(question_type)),
            }
        } else {
            Ok(JustClickNext)
        }
    } else {
        Ok(Fuckd)
    }
}
