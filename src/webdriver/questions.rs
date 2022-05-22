// Imma just chuck this selecotr here for later
//
// Selector for selecting the underlined elements in the "You fucked up text box"

use crate::delay;
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, WebDriver};
use tracing::debug;

pub async fn start_intro(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .find_element(By::Css(r#"a[data-test="intro-lesson"]"#))
        .await?
        .click()
        .await?;

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

pub async fn choose_answer_underline_test(
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
