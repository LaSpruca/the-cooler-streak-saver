// Imma just chuck this selecotr here for later
// [data-test="blame blame-incorrect"] > div > div > div > div > span > span[class]
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
            return check_answer(&driver).await;
        }
    }

    // Get the correct answer for gods sake
    Ok(Some(skip(driver).await?))
}

pub async fn type_translation(
    driver: &WebDriver,
    correct_answer: String,
) -> WebDriverResult<Option<String>> {
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

    check_answer(&driver).await
}

async fn check_answer(driver: &WebDriver) -> WebDriverResult<Option<String>> {
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
