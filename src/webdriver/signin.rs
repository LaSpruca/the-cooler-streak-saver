use super::Error;
use std::env;
use std::time::Duration;
use thirtyfour::error::WebDriverError;
use thirtyfour::{By, WebDriver};
use tokio::time::{Instant, sleep};
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
        .find_element(By::Css(r#"input[data-test="password-input"]"#))
        .await?
        .send_keys(&password)
        .await?;

    // Click the login button
    driver
        .find_element(By::Css(r#"button[data-test="register-button"]"#))
        .await?
        .click()
        .await?;

    // Give duolingo max 20 seconds to login
    let time_elapsed = Instant::now();
    let timout = Duration::from_secs(20);
    debug!("Logging In...");

    // While either not on the learn page, or can't find the skill tree
    while driver.current_url().await? != "https://www.duolingo.com/learn"
        || driver
            .find_element(By::Css("div[data-test=\"skill-tree\"]"))
            .await
            .is_err()
    {
        if time_elapsed.elapsed() > timout {
            break;
        }
        sleep(Duration::from_millis(100)).await;

    }
    debug!("Logged In");

    // Checks for "welcome back" message and dismisses it
    let no_thanks_button = driver.find_element(By::Css("button[data-test=\"notification-drawer-no-thanks-button\"]")).await;
    if no_thanks_button.is_ok() {
        debug!("Found welcome back message, dismissing");
        no_thanks_button.unwrap().click().await?;
        sleep(Duration::from_millis(200)).await;
    }

    // Check to see if login is successful
    if !driver.current_url().await?.ends_with("learn") {
        match driver
            .find_element(By::Css(r#"div[data-test="invalid-form-field"]"#))
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
