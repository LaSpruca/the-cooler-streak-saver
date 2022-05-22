// Imma just chuck this selecotr here for later
//
// Selector for selecting the underlined elements in the "You fucked up text box"

use crate::delay;
use std::collections::HashMap;
use thirtyfour::common::capabilities::firefox::LogLevel::Error;
use thirtyfour::error::{WebDriverError, WebDriverErrorInfo, WebDriverResult};
use thirtyfour::{By, ElementId, WebDriver, WebElement};
use tracing::debug;

pub async fn start_intro(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .find_element(By::Css(r#"a[data-test="intro-lesson"]"#))
        .await?
        .click()
        .await?;

    Ok(())
}

pub async fn next_skill_tree_item(driver: &WebDriver) -> WebDriverResult<()> {
    let all_lesson_buttons = driver
        .find_elements(By::Css(r#"div[data-test="skill"]>div[tabindex]"#))
        .await?;

    let mut started_lesson = false;

    for lesson_button in all_lesson_buttons {
        lesson_button.click().await?;
        debug!("Clicking on skill");
        delay!(500);

        let get_start_buttons = r#"a[data-test="start-button"]"#;
        while driver
            .find_element(By::Css(get_start_buttons))
            .await
            .is_err()
        {
            lesson_button.click().await?;
            delay!(500);
        }

        let start_button = driver.find_element(By::Css(get_start_buttons)).await?;

        debug!("{}", start_button);
        if !start_button.text().await?.contains("PRACTICE") {
            // Let's start the lesson
            start_button.click().await?;
            started_lesson = true;
            break;
        }
        // Done lesson, so we can close it and continue onwards
        lesson_button.click().await?;
    }
    assert!(
        started_lesson,
        "Failed to start lesson - nothing left to complete"
    );

    Ok(())
}

/// Skips a question, consumes all text in the blame-incorrect section as answer
pub async fn skip(driver: &WebDriver) -> WebDriverResult<String> {
    driver
        .find_element(By::Css(r#"button[data-test="player-skip"]"#))
        .await?
        .click()
        .await?;

    let correct = driver
        .find_element(By::Css(
            r#"[data-test="blame blame-incorrect"] > div > div> div > div"#,
        ))
        .await?
        .text()
        .await?;

    click_next(driver).await?;

    Ok(correct)
}

/// Skips a question, consumes only underlined text in the blame-incorrect section as answer
pub async fn skip_underline(driver: &WebDriver) -> WebDriverResult<String> {
    driver
        .find_element(By::Css(r#"button[data-test="player-skip"]"#))
        .await?
        .click()
        .await?;

    let correct = driver
        .find_element(By::Css(
            r#"[data-test="blame blame-incorrect"] > div > div > div > div > span > span[class]"#,
        ))
        .await?
        .text()
        .await?;

    click_next(driver).await?;

    Ok(correct)
}

pub async fn click_next(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .find_element(By::Css(r#"button[data-test="player-next"]"#))
        .await?
        .click()
        .await?;

    Ok(())
}

pub async fn choose_answer(
    driver: &WebDriver,
    correct_answer: String,
) -> WebDriverResult<Option<String>> {
    let possibles = driver
        .find_elements(By::Css(
            r#"[data-test="challenge-choice"] > div > span[dir]"#,
        ))
        .await?;
    debug!("Found element");

    for possible in possibles {
        if possible.text().await? == correct_answer {
            debug!("Got correct");

            possible.click().await?;

            // Check to see if the question was answered correctly
            return check_answer_full(&driver).await;
        }
    }

    debug!("Could not find correct answer");

    // Get the correct answer for gods sake
    Ok(Some(skip(driver).await?))
}

pub async fn choose_answer_assist(
    driver: &WebDriver,
    correct_answer: String,
) -> WebDriverResult<Option<String>> {
    let possibles = driver
        .find_elements(By::Css(r#"[data-test="challenge-choice"] > div"#))
        .await?;
    debug!("Found element");

    for possible in possibles {
        if possible.text().await? == correct_answer {
            debug!("Got correct");

            possible.click().await?;

            // Check to see if the question was answered correctly
            return check_answer_full(&driver).await;
        }
    }

    debug!("Could not find correct answer");

    // Get the correct answer for gods sake
    Ok(Some(skip(driver).await?))
}

pub async fn choose_answer_underline_test(
    driver: &WebDriver,
    correct_answer: String,
) -> WebDriverResult<Option<String>> {
    let possibles = driver
        .find_elements(By::Css(r#"div > [data-test="challenge-tap-token"] > span"#))
        .await?;
    debug!("Found element");

    for possible in possibles {
        debug!("{}", possible.text().await?);
        if possible.text().await? == correct_answer {
            debug!("Got correct");

            possible.click().await?;

            // Check to see if the question was answered correctly
            return check_answer_underline(&driver).await;
        }
    }

    debug!("Could not find correct answer");

    // Get the correct answer for gods sake
    Ok(Some(skip_underline(driver).await?))
}

pub async fn type_translation(
    driver: &WebDriver,
    correct_answer: String,
) -> WebDriverResult<Option<String>> {
    delay!(100);
    let keybd_button = driver
        .find_element(By::Css(r#"[data-test="player-toggle-keyboard"]"#))
        .await?;

    debug!("{}", keybd_button.text().await?);
    if keybd_button.text().await?.to_lowercase() == "use keyboard" {
        keybd_button.click().await?;
    }

    driver
        .find_element(By::Css(r#"[data-test="challenge-translate-input"]"#))
        .await?
        .send_keys(correct_answer)
        .await?;

    check_answer_full(&driver).await
}

/// Click the "check" and checks to see if the answer was correct, if it is incorrect, the full
/// answer is returned
async fn check_answer_full(driver: &WebDriver) -> WebDriverResult<Option<String>> {
    click_next(&driver).await?;
    delay!(500);

    let result = if let Ok(correct_display) = driver
        .find_element(By::Css(
            r#"[data-test="blame blame-incorrect"] > div > div> div > div"#,
        ))
        .await
    {
        Some(correct_display.text().await?)
    } else {
        None
    };

    click_next(&driver).await?;

    Ok(result)
}

/// Click the "check" and checks to see if the answer was correct, if it is incorrect, only the
/// underlined component of the answer is returned.
async fn check_answer_underline(driver: &WebDriver) -> WebDriverResult<Option<String>> {
    click_next(&driver).await?;
    delay!(500);

    let result = if let Ok(correct_display) = driver
        .find_element(By::Css(
            r#"[data-test="blame blame-incorrect"] > div > div > div > div > span > span[class]"#,
        ))
        .await
    {
        Some(correct_display.text().await?)
    } else {
        None
    };

    click_next(&driver).await?;

    Ok(result)
}

pub async fn ignore_question(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .find_element(By::Css(r#"button[data-test="player-skip"]"#))
        .await?
        .click()
        .await?;

    click_next(driver).await?;

    Ok(())
}

pub async fn answer_match(
    driver: &WebDriver,
    questions: &HashMap<String, Option<String>>,
) -> WebDriverResult<HashMap<String, String>> {
    let mut response = HashMap::new();

    for (question, answer) in questions.iter() {
        if let Some(answer) = answer {
            if !select_pair(driver, question, answer).await? {
                response.insert(question.clone(), brute_force(driver, question).await?);
            }
        } else {
            response.insert(question.clone(), brute_force(driver, question).await?);
        }
    }
    click_next(driver).await?;

    Ok(response)
}

async fn brute_force(driver: &WebDriver, question: &String) -> WebDriverResult<String> {
    for element in driver.find_elements(By::Css(r#"[data-test="challenge challenge-match"] > div > div > div > div > div:nth-child(2) > div > button"#)).await? {
        if element.get_attribute("aria-disabled").await?.is_none() {
            let text = element.text().await?;
            let other_text = element.find_element(By::Tag("span")).await?.text().await?;
            let answer = text.strip_prefix(other_text.as_str()).unwrap().strip_prefix("\n").unwrap();
            if select_pair(driver, question, answer).await? {
                return Ok(answer.to_string());
                delay!(500);
            }
            // Select Pair takes 800ms for animation
            delay!(1000);
        }
    }

    Err(WebDriverError::NoSuchElement(WebDriverErrorInfo::new(
        &format!("Could not find answer for {question}"),
    )))
}

async fn select_pair(driver: &WebDriver, question: &str, answer: &str) -> WebDriverResult<bool> {
    select_multi(driver, question, true).await?;
    let elm = select_multi(driver, answer, false).await?;

    Ok(
        if let Some(disabled) = elm.get_attribute("aria-disabled").await? {
            true
        } else {
            false
        },
    )
}

async fn select_multi<'a>(
    webdriver: &'a WebDriver,
    text: &str,
    left: bool,
) -> WebDriverResult<WebElement<'a>> {
    let selector = format!(
        r#"[data-test="challenge challenge-match"] > div > div > div > div > div:nth-child({}) > div > button"#,
        if left { 1 } else { 2 }
    );
    for elm in webdriver.find_elements(By::Css(selector.as_str())).await? {
        if elm.text().await?.ends_with(text) {
            elm.click().await?;
            return Ok(elm);
        }
    }

    return Err(WebDriverError::NoSuchElement(WebDriverErrorInfo::new(
        "¯\\_(ツ)_/¯",
    )));
}
