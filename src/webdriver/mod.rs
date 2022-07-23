use thirtyfour_sync::WebDriver;

mod get_state;
mod questions;
mod signin;

pub use get_state::*;
pub use questions::*;
pub use signin::*;

use std::env;
use thirtyfour_sync::{error::WebDriverResult, prelude::*};
use tracing::info;

use crate::common::QuestionType;

pub fn open_browser() -> WebDriverResult<WebDriver> {
    info!("Creating connection");

    let mut caps = DesiredCapabilities::chrome();

    // Disable notification popup
    caps.add_chrome_arg("--disable-notifications")?;

    if let Ok(chrome_path) = env::var("CHROME_PATH") {
        info!("Using chrome path {chrome_path}");
        caps.set_binary(&chrome_path)?;
    }

    if let Ok(headless) = env::var("HEADLESS") {
        if headless.trim() == "true" || headless.trim() == "1" {
            info!("Enabling headless mode");
            caps.add_chrome_arg("--no-sandbox")?;
            caps.add_chrome_arg("--disable-dev-shm-usage")?;
            caps.add_chrome_arg("--window-size=1920,1200")?;
        }
    }

    let driver = WebDriver::new(
        &env::var("DRIVER_URL").unwrap_or("http://localhost:4444".into()),
        &caps,
    )?;

    // Remove all alerts
    driver.execute_script("window.alert = function() {};")?;
    driver.execute_script("window.onbeforeunload = function() {};")?;

    Ok(driver)
}

pub fn answer_question(
    driver: &WebDriver,
    question_type: QuestionType,
    ans: String,
) -> WebDriverResult<Option<String>> {
    match question_type {
        QuestionType::Translate => type_translation(&driver, ans),
        QuestionType::Select => choose_answer(&driver, ans),
        QuestionType::Assist => choose_answer_assist(&driver, ans),
        QuestionType::TapComplete => choose_answer_underline_test(&driver, ans),
        QuestionType::Name => type_translation(&driver, ans),
        QuestionType::MatchPairs => {
            unreachable!()
        }
        QuestionType::CompleteTranslation => type_translation_complete(&driver, ans),
        QuestionType::GapFill => {
            todo!()
        }
    }
}
