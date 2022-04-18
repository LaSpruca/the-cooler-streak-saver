use std::time::Duration;
use thirtyfour::error::WebDriverResult;
use thirtyfour::{By, WebDriver};

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

    tokio::time::sleep(Duration::from_secs(2)).await;

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

// pub async fn choose_answer(driver: &WebDriver, correct_answer: String) -> WebDriverResult<()> {}
