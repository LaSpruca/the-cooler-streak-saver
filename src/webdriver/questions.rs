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
        .find_element(By::Css(r#"div[data-test*="blame blame-incorrect"]"#))
        .await?
        .find_element(By::XPath("/div[2]/div[1]/div/div"))
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
