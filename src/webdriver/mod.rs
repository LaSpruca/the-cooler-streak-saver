mod get_state;
mod questions;
mod signin;

pub use crate::webdriver::get_state::State;
use crate::webdriver::questions::*;
use crate::webdriver::signin::browser_login;
use crate::{delay, QuestionType};
use get_state::get_state as driver_get_state;
use std::collections::HashMap;
use std::env;
use thirtyfour::error::WebDriverError;
use thirtyfour::prelude::*;
use thiserror::Error as ThisError;
use tokio::sync::mpsc::{channel, Sender};
use tracing::info;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("WebDriverError {0}")]
    WebDriverError(WebDriverError),
    #[error("Could not get Duoling username")]
    NoUsername,
    #[error("Could not get Duolingo password")]
    NoPassword,
    #[error("There is no account with the username \"{0}\"")]
    InvalidUsername(String),
    #[error("Could not sign into \"{0}\" with the provided password")]
    InvalidPassword(String),
    #[error("I think the bird is having a stroke")]
    ThereIsAnErrorForSomeReason,
    #[error("Unexpected driver response")]
    UnexpectedDriverResponse(Box<Response>),
    #[error("No driver response")]
    NoDriverResponse,
}

impl From<WebDriverError> for Error {
    fn from(ex: WebDriverError) -> Self {
        Self::WebDriverError(ex)
    }
}

#[derive(Debug)]
pub enum Signal {
    Quit,
    SignIn,
    GetSate,
    StartLanguage,
    StartLesson,
    Skip,
    SkipUnderlined,
    ClickNext,
    AnswerQuestion(String, QuestionType),
    IgnoreQuestion,
    MultiAnswerQuestion(HashMap<String, Option<String>>),
    YeetDuoMarketing,
    HereIsATip
}

#[derive(Debug)]
pub enum Response {
    Exited,
    SignInSuccess,
    SignInError(Error),
    StateResponse(State),
    WebDriverError(WebDriverError),
    Success,
    SkipResponse(String),
    AnswerResponse(Option<String>),
    MultiAnswerResponse(HashMap<String, String>),
}

impl Response {
    pub fn is_exited(&self) -> bool {
        return if let Self::Exited = self { true } else { false };
    }
}

pub type WebdriverSender = Sender<(Signal, Sender<Response>)>;

pub async fn open_browser() -> WebDriverResult<WebdriverSender> {
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
    )
    .await?;

    // Remove all alerts
    driver.execute_script("window.alert = function() {};").await?;
    driver.execute_script("window.onbeforeunload = function() {};").await?;

    let (signal_in, mut signal_out) = channel::<(Signal, Sender<Response>)>(25);

    tokio::spawn(async move {
        while let Some((signal, sender)) = signal_out.recv().await {
            match signal {
                Signal::Quit => {
                    info!("Committing yesn't");
                    driver.quit().await.unwrap();
                    sender.send(Response::Exited).await.unwrap();
                    break;
                }
                Signal::SignIn => {
                    match browser_login(&driver).await {
                        Ok(_) => sender.send(Response::SignInSuccess).await.unwrap(),
                        Err(ex) => sender.send(Response::SignInError(ex)).await.unwrap(),
                    };
                }
                Signal::GetSate => {
                    match driver_get_state(&driver).await {
                        Ok(val) => sender.send(Response::StateResponse(val)).await.unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    };
                }
                Signal::StartLanguage => {
                    match start_intro(&driver).await {
                        Ok(_) => sender.send(Response::Success).await.unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    };
                }
                Signal::StartLesson => {
                    match next_skill_tree_item(&driver).await {
                        Ok(_) => sender.send(Response::Success).await.unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    };
                }
                Signal::Skip => {
                    match skip(&driver).await {
                        Ok(answer) => sender.send(Response::SkipResponse(answer)).await.unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    };
                }
                Signal::SkipUnderlined => {
                    match skip_underline(&driver).await {
                        Ok(answer) => sender.send(Response::SkipResponse(answer)).await.unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    };
                }

                Signal::ClickNext => {
                    match click_next(&driver).await {
                        Ok(_) => sender.send(Response::Success).await.unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    };
                }
                Signal::AnswerQuestion(ans, question_type) => {
                    let res = match question_type {
                        QuestionType::Translate => type_translation(&driver, ans).await,
                        QuestionType::Select => choose_answer(&driver, ans).await,
                        QuestionType::Assist => choose_answer_assist(&driver, ans).await,
                        QuestionType::TapComplete => {
                            choose_answer_underline_test(&driver, ans).await
                        }
                        QuestionType::Name => type_translation(&driver, ans).await,
                        QuestionType::MatchPairs => {
                            unreachable!()
                        }
                        QuestionType::CompleteTranslation => {
                            type_translation_complete(&driver, ans).await
                        }
                    };

                    match res {
                        Ok(opt) => sender.send(Response::AnswerResponse(opt)).await.unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    };
                }
                Signal::IgnoreQuestion => {
                    match ignore_question(&driver).await {
                        Ok(_) => sender.send(Response::Success).await.unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    };
                }
                Signal::MultiAnswerQuestion(answers) => {
                    match answer_match(&driver, &answers).await {
                        Ok(val) => sender
                            .send(Response::MultiAnswerResponse(val))
                            .await
                            .unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    }
                }
                Signal::YeetDuoMarketing => {
                    match click_nothanks(&driver).await {
                        Ok(_) => sender.send(Response::Success).await.unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    };
                }
                Signal::HereIsATip => {
                    match here_is_tip(&driver).await {
                        Ok(_) => sender.send(Response::Success).await.unwrap(),
                        Err(ex) => sender.send(Response::WebDriverError(ex)).await.unwrap(),
                    };
                }
            }
        }
    });

    Ok(signal_in)
}

pub async fn quit(tx: WebdriverSender) {
    let (res_tx, mut res_rx) = channel(2);

    tx.send((Signal::Quit, res_tx)).await.unwrap();
    while let Some(res) = res_rx.recv().await {
        info!("{res:?}");
        if res.is_exited() {
            break;
        }
    }
}

pub async fn sign_in(tx: &WebdriverSender) -> Result<(), Error> {
    let (res_tx, mut rx) = channel(2);
    tx.send((Signal::SignIn, res_tx)).await.unwrap();
    match rx.recv().await {
        Some(signal) => match signal {
            Response::SignInSuccess => Ok(()),
            Response::SignInError(ex) => Err(ex),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}

pub async fn get_state(tx: &WebdriverSender) -> Result<State, Error> {
    let (res_tx, mut rx) = channel(2);
    tx.send((Signal::GetSate, res_tx)).await.unwrap();
    match rx.recv().await {
        Some(signal) => match signal {
            Response::StateResponse(result) => Ok(result),
            Response::WebDriverError(ex) => Err(Error::WebDriverError(ex)),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}

pub async fn start_language(tx: &WebdriverSender) -> Result<(), Error> {
    let (res_tx, mut rx) = channel(2);
    tx.send((Signal::StartLanguage, res_tx)).await.unwrap();
    match rx.recv().await {
        Some(signal) => match signal {
            Response::Success => {
                delay!(5000);
                Ok(())
            }
            Response::WebDriverError(ex) => Err(Error::WebDriverError(ex)),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}

pub async fn answer_here_is_a_tip(tx: &WebdriverSender) -> Result<(), Error> {
    let (res_tx, mut rx) = channel(2);
    tx.send((Signal::HereIsATip, res_tx)).await.unwrap();
    match rx.recv().await {
        Some(signal) => match signal {
            Response::Success => {
                delay!(5000);
                Ok(())
            }
            Response::WebDriverError(ex) => Err(Error::WebDriverError(ex)),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}

pub async fn start_lesson(tx: &WebdriverSender) -> Result<(), Error> {
    let (res_tx, mut rx) = channel(2);
    tx.send((Signal::StartLesson, res_tx)).await.unwrap();
    match rx.recv().await {
        Some(signal) => match signal {
            Response::Success => {
                delay!(5000);
                Ok(())
            }
            Response::WebDriverError(ex) => Err(Error::WebDriverError(ex)),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}

pub async fn skip_question(
    tx: &WebdriverSender,
    question_type: QuestionType,
) -> Result<String, Error> {
    let (res_tx, mut rx) = channel(2);
    if question_type.check_underline() {
        tx.send((Signal::SkipUnderlined, res_tx)).await.unwrap();
    } else {
        tx.send((Signal::Skip, res_tx)).await.unwrap();
    }
    match rx.recv().await {
        Some(signal) => match signal {
            Response::SkipResponse(answer) => {
                delay!(500);
                Ok(answer)
            }
            Response::WebDriverError(ex) => Err(Error::WebDriverError(ex)),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}

pub async fn next(tx: &WebdriverSender) -> Result<(), Error> {
    let (res_tx, mut rx) = channel(2);
    tx.send((Signal::ClickNext, res_tx)).await.unwrap();
    match rx.recv().await {
        Some(signal) => match signal {
            Response::Success => {
                delay!(500);
                Ok(())
            }
            Response::WebDriverError(ex) => Err(Error::WebDriverError(ex)),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}

pub async fn discard_question(tx: &WebdriverSender) -> Result<(), Error> {
    let (res_tx, mut rx) = channel(2);
    tx.send((Signal::IgnoreQuestion, res_tx)).await.unwrap();
    match rx.recv().await {
        Some(signal) => match signal {
            Response::Success => {
                delay!(500);
                Ok(())
            }
            Response::WebDriverError(ex) => Err(Error::WebDriverError(ex)),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}

pub async fn answer_question(
    tx: &WebdriverSender,
    answer: String,
    question_type: QuestionType,
) -> Result<Option<String>, Error> {
    let (res_tx, mut rx) = channel(2);
    tx.send((Signal::AnswerQuestion(answer, question_type), res_tx))
        .await
        .unwrap();
    match rx.recv().await {
        Some(signal) => match signal {
            Response::AnswerResponse(res) => {
                delay!(500);
                Ok(res)
            }
            Response::WebDriverError(ex) => Err(Error::WebDriverError(ex)),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}

pub async fn answer_multi_question(
    tx: &WebdriverSender,
    answers: HashMap<String, Option<String>>,
) -> Result<HashMap<String, String>, Error> {
    let (res_tx, mut rx) = channel(2);
    tx.send((Signal::MultiAnswerQuestion(answers), res_tx))
        .await
        .unwrap();
    match rx.recv().await {
        Some(signal) => match signal {
            Response::MultiAnswerResponse(res) => {
                delay!(500);
                Ok(res)
            }
            Response::WebDriverError(ex) => Err(Error::WebDriverError(ex)),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}

pub async fn yeet_duo_marking(tx: &WebdriverSender) -> Result<(), Error> {
    let (res_tx, mut rx) = channel(2);
    tx.send((Signal::YeetDuoMarketing, res_tx)).await.unwrap();
    match rx.recv().await {
        Some(signal) => match signal {
            Response::Success => {
                delay!(500);
                Ok(())
            }
            Response::WebDriverError(ex) => Err(Error::WebDriverError(ex)),
            _ => Err(Error::UnexpectedDriverResponse(Box::new(signal))),
        },
        None => Err(Error::NoDriverResponse),
    }
}
