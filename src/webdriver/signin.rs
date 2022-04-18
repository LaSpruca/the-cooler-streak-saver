use super::Error;
use std::env;
use std::time::Duration;
use thirtyfour::error::WebDriverError;
use thirtyfour::{By, WebDriver};
use tracing::debug;

pub async fn browser_login(driver: &WebDriver) -> Result<(), Error> {
    // Get the username and password from env vars
    let username = match env::var("DUOLINGO_USERNAME") {
        Ok(val) => val,
        Err(_) => return Err(Error::NoUsername),
    };
    let password = match env::var("DUOLINGO_PASSWORD") {
        Ok(val) => val,
        Err(_) => return Err(Error::NoPassword),
    };

    // Go to duolingo.com
    driver.get("https://duolingo.com").await?;

    // Click the login button
    driver
        .find_element(By::Css("button[data-test=\"have-account\"]"))
        .await?
        .click()
        .await?;

    // Input the email
    driver
        .find_element(By::Css("input[data-test=\"email-input\"]"))
        .await?
        .send_keys(&username)
        .await?;

    // Input the password
    driver
        .find_element(By::Css("input[data-test=\"password-input\"]"))
        .await?
        .send_keys(&password)
        .await?;

    // Click the login button
    driver
        .find_element(By::Css("button[data-test=\"register-button\"]"))
        .await?
        .click()
        .await?;

    // Give duolingo 5 seconds to login
    tokio::time::sleep(Duration::from_secs(5)).await;

    debug!("{}", driver.current_url().await?);

    // Check to see if login is successfull
    if !driver.current_url().await?.ends_with("learn") {
        match driver
            .find_element(By::Css("div[data-test=\"invalid-form-field\"]"))
            .await
        {
            Ok(div) => {
                return if div.text().await?.contains("Duolingo account") {
                    Err(Error::InvalidUsername(username))
                } else if div.text().await?.contains("password") {
                    Err(Error::InvalidPassword(username))
                } else {
                    Err(Error::ThereIsAnErrorForSomeReason)
                };
            }
            Err(err) => match err {
                WebDriverError::NoSuchElement(_) => {}
                _ => {
                    return Err(Error::ThereIsAnErrorForSomeReason);
                }
            },
        }
    }

    Ok(())
}
