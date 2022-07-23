use std::fmt::Display;
use std::time::Duration;
use std::{env, time::Instant};
use thirtyfour_sync::error::WebDriverError;
use thirtyfour_sync::{By, WebDriver, WebDriverCommands};
use tracing::debug;

#[derive(thiserror::Error, Debug)]
pub enum SignInError {
    NoUsername,
    NoPassword,
    WebDriverError(WebDriverError),
    InvalidUsername,
    InvalidPassword,
    UnknownError,
}

impl Display for SignInError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignInError::WebDriverError(ex) => write!(f, "{ex}"),
            SignInError::InvalidUsername => write!(f, "The username is invalid"),
            SignInError::InvalidPassword => write!(f, "The password is invalid"),
            SignInError::UnknownError => write!(f, "There was an unknown error"),
            SignInError::NoUsername => write!(f, "There is no username"),
            SignInError::NoPassword => write!(f, "There is no password"),
        }
    }
}

impl From<WebDriverError> for SignInError {
    fn from(val: WebDriverError) -> Self {
        Self::WebDriverError(val)
    }
}

pub fn browser_login(driver: &WebDriver) -> Result<(), SignInError> {
    // Get the username and password from env vars
    let username = match env::var("DUOLINGO_USERNAME") {
        Ok(val) => val,
        Err(_) => return Err(SignInError::NoUsername),
    };
    let password = match env::var("DUOLINGO_PASSWORD") {
        Ok(val) => val,
        Err(_) => return Err(SignInError::NoPassword),
    };

    // Go to duolingo.com
    driver.get("https://duolingo.com")?;

    // Click the login button
    driver
        .find_element(By::Css("button[data-test=\"have-account\"]"))?
        .click()?;

    // Input the email
    driver
        .find_element(By::Css("input[data-test=\"email-input\"]"))?
        .send_keys(&username)?;

    // Input the password
    driver
        .find_element(By::Css(r#"input[data-test="password-input"]"#))?
        .send_keys(&password)?;

    // Click the login button
    driver
        .find_element(By::Css(r#"button[data-test="register-button"]"#))?
        .click()?;

    // Give duolingo max 20 seconds to login
    let time_elapsed = Instant::now();
    let timout = Duration::from_secs(20);
    debug!("Logging In...");

    // While either not on the learn page, or can't find the skill tree
    while driver.current_url()? != "https://www.duolingo.com/learn"
        || driver
            .find_element(By::Css("div[data-test=\"skill-tree\"]"))
            .is_err()
    {
        if time_elapsed.elapsed() > timout {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    debug!("Logged In");

    // Check to see if login is successful
    if !driver.current_url()?.ends_with("learn") {
        match driver.find_element(By::Css(r#"div[data-test="invalid-form-field"]"#)) {
            Ok(div) => {
                return if div.text()?.contains("Duolingo account") {
                    Err(SignInError::InvalidUsername)
                } else if div.text()?.contains("password") {
                    Err(SignInError::InvalidPassword)
                } else {
                    Err(SignInError::UnknownError)
                };
            }
            Err(err) => match err {
                WebDriverError::NoSuchElement(_) => {}
                _ => {
                    return Err(SignInError::UnknownError);
                }
            },
        }
    }

    Ok(())
}
