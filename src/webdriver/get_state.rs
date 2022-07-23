use crate::common::QuestionType;
use crate::delay;
use crate::webdriver::get_state::State::{Fuckd, JustClickNext, Question, UnknownQuestionType};
use crate::State::{Legendary, Loading, PlusScreen};
use thirtyfour_sync::{error::WebDriverResult, By, WebDriver, WebDriverCommands};
use tracing::{debug, info};

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    StartLanguage,
    StartLesson,
    /// - `0`: The kind of question
    /// - `1`: The language
    /// - `2`: The question itself
    Question(QuestionType, String, String),
    // EndLessonSlide,
    /// This fucker is special so it gets its own enum variant
    MatchQuestion(Vec<String>, String),
    JustClickNext,
    Fuckd,
    UnknownQuestionType(String),
    IgnoreQuestion,
    PlusScreen,
    HereIsATip,
    Loading,
    Legendary,
}

pub fn get_state(driver: &WebDriver) -> WebDriverResult<State> {
    // Check to see if on skill tree page
    if let Ok(elm) = driver.find_element(By::Css("div[data-test=\"skill-tree\"]")) {
        debug!("Found skill tree");
        if let Ok(_) = elm.find_element(By::Css("h2")) {
            Ok(State::StartLanguage)
        } else {
            debug!("Starting Lesson!");
            Ok(State::StartLesson)
        }
    }
    // Check if there is a next button
    else if let Ok(btn) = driver.find_element(By::Css(r#"button[data-test="player-next"]"#)) {
        // Check to see if the loading button is going brrr
        if let Ok(_) = btn.find_element(By::Css("div")) {
            return Ok(Loading);
        }

        debug!("Found next button");
        // Check for the challenge div
        if let Ok(question) = driver.find_element(By::Css(r#"div[data-test*="challenge"]"#)) {
            debug!("Found challenge div");

            // Get the question type from the data-test attribute
            let question_type = question
                .get_attribute("data-test")?
                .unwrap_or(String::new())
                .strip_prefix("challenge ")
                .unwrap_or("no-question-data-attribute")
                .to_string();

            debug!("Found got question type");

            let language = driver
                .current_url()?
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
                        .find_element(By::Css(r#"h1[data-test="challenge-header"]"#))?
                        .text()?;
                    Ok(Question(QuestionType::Select, language, text))
                }

                "challenge-translate" => {
                    let text = driver
                        .find_element(By::Css(r#"[data-test="challenge challenge-translate"] > div > div > div > div> div> div > div"#))
                        ?
                        .text()
                        ?;
                    Ok(Question(QuestionType::Translate, language, text))
                }

                "challenge-assist" => {
                    let text = driver
                        .find_element(By::Css(r#"h1[data-test="challenge-header"]"#))?
                        .text()?;
                    Ok(Question(QuestionType::Assist, language, text))
                }

                "challenge-tapComplete" => {
                    let mut text = String::new();
                    for element in driver.find_elements(By::Css(
                        r#"[data-test="challenge challenge-tapComplete"] > div > div > div > span"#,
                    ))? {
                        let elm_text = element.text()?;
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
                    driver
                        .find_element(By::Css(r#"[data-test="player-toggle-keyboard"]"#))?
                        .click()?;

                    while driver
                        .find_element(By::Css(r#"[data-test="challenge-translate-input"]"#))
                        .is_err()
                    {
                        delay!(500);
                        let make_harder_text = driver
                            .find_element(By::Css(r#"[data-test="player-toggle-keyboard"]"#))?
                            .text()?;

                        if make_harder_text.contains("HARDER") {
                            driver
                                .find_element(By::Css(r#"[data-test="player-toggle-keyboard"]"#))?
                                .click()?;
                        }
                    }

                    let text = driver
                        .find_element(By::Css(r#"[data-test="challenge challenge-completeReverseTranslation"] > div > div > div > div > div > div > span"#))
                        ?
                        .text()
                        ?;
                    Ok(Question(QuestionType::CompleteTranslation, language, text))
                }

                "challenge-match" => {
                    let mut questions = vec![];

                    // Get all of the listed questions
                    for element in driver
                        .find_elements(
                            By::Css(r#"[data-test="challenge challenge-match"] > div > div > div > div > div:nth-child(1) > div > button"#)
                        )
                        ?
                        .into_iter() {

                        let text = element.text()?;
                        let span_text = element
                            .find_element(By::Tag("span"))
                            ?
                            .text()
                            ?;

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

                "challenge-listenTap"
                | "challenge-listen"
                | "challenge-speak"
                | "challenge-listenComplete" => return Ok(State::IgnoreQuestion),

                "challenge-name" => {
                    let text = driver
                        .find_element(By::Css(r#"h1[data-test="challenge-header"]"#))?
                        .text()?;

                    Ok(Question(QuestionType::Name, language, text))
                }

                "challenge-gapFill" => {
                    let text = driver
                        .find_element(By::Css(
                            r#"[data-test="challenge challenge-gapFill"] > div > div> div"#,
                        ))?
                        .text()?;

                    Ok(Question(QuestionType::GapFill, language, text))
                }

                "no-question-data-attribute" => {
                    // Currently, I think only "Here's a tip" does this

                    Ok(State::HereIsATip)
                }

                _ => Ok(UnknownQuestionType(question_type)),
            }
        } else {
            Ok(JustClickNext)
        }
    } else if let Ok(_) = driver.find_element(By::Css(r#"button[data-test="plus-no-thanks"]"#)) {
        Ok(PlusScreen)
    } else if let Ok(_) = driver.find_element(By::Css(r#"[data-test="final-level-promo"]"#)) {
        Ok(Legendary)
    } else {
        Ok(Fuckd)
    }
}
