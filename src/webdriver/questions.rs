use crate::delay;
use std::collections::HashMap;
use thirtyfour_sync::{
    error::{WebDriverError, WebDriverErrorInfo, WebDriverResult},
    By, WebDriver, WebDriverCommands, WebElement,
};
use tracing::debug;

pub fn start_intro(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .find_element(By::Css(r#"a[data-test="intro-lesson"]"#))?
        .click()?;

    Ok(())
}

pub fn next_skill_tree_item(driver: &WebDriver) -> WebDriverResult<()> {
    let all_lesson_buttons =
        driver.find_elements(By::Css(r#"div[data-test="skill"]>div[tabindex]"#))?;

    let mut started_lesson = false;

    for lesson_button in all_lesson_buttons {
        lesson_button.click()?;
        debug!("Clicking on skill");
        delay!(500);

        let get_start_buttons = r#"a[data-test="start-button"]"#;
        while driver.find_element(By::Css(get_start_buttons)).is_err() {
            lesson_button.click()?;
            delay!(500);
        }

        let start_button = driver.find_element(By::Css(get_start_buttons))?;

        debug!("{}", start_button);
        if !start_button.text()?.contains("PRACTICE") {
            // Let's start the lesson
            start_button.click()?;
            started_lesson = true;
            break;
        }
        // Done lesson, so we can close it and continue onwards
        lesson_button.click()?;
    }
    assert!(
        started_lesson,
        "Failed to start lesson - nothing left to complete"
    );

    Ok(())
}

/// Skips a question, consumes all text in the blame-incorrect section as answer
pub fn skip(driver: &WebDriver) -> WebDriverResult<String> {
    driver
        .find_element(By::Css(r#"button[data-test="player-skip"]"#))?
        .click()?;

    delay!(500);

    let correct = get_answer_text(&driver)?.unwrap();

    click_next(driver)?;

    Ok(correct)
}

/// Skips a question, consumes only underlined text in the blame-incorrect section as answer
pub fn skip_underline(driver: &WebDriver) -> WebDriverResult<String> {
    driver
        .find_element(By::Css(r#"button[data-test="player-skip"]"#))?
        .click()?;

    let mut correct = vec![];
    for elm in driver.find_elements(By::Css(
        r#"[data-test="blame blame-incorrect"] > div > div > div > div > span > span[class]"#,
    ))? {
        correct.push(elm.text()?);
    }

    click_next(driver)?;

    Ok(correct.join(","))
}

pub fn click_next(driver: &WebDriver) -> WebDriverResult<()> {
    debug!("Clicking Next!");
    driver
        .find_element(By::Css(r#"button[data-test="player-next"]"#))?
        .click()?;
    debug!("Clicked Next!");

    Ok(())
}

pub fn choose_answer(
    driver: &WebDriver,
    correct_answer: String,
) -> WebDriverResult<Option<String>> {
    let possibles = driver.find_elements(By::Css(
        r#"[data-test="challenge-choice"] > div > span[dir]"#,
    ))?;
    debug!("Found element");

    for possible in possibles {
        if possible.text()? == correct_answer {
            debug!("Got correct");

            possible.click()?;

            // Check to see if the question was answered correctly
            return check_answer_full(&driver);
        }
    }

    debug!("Could not find correct answer");

    // Get the correct answer for gods sake
    Ok(Some(skip(driver)?))
}

pub fn choose_answer_assist(
    driver: &WebDriver,
    correct_answer: String,
) -> WebDriverResult<Option<String>> {
    let possibles = driver.find_elements(By::Css(r#"[data-test="challenge-choice"] > div"#))?;
    debug!("Found element");

    for possible in possibles {
        if possible.text()? == correct_answer {
            debug!("Got correct");

            possible.click()?;

            // Check to see if the question was answered correctly
            return check_answer_full(&driver);
        }
    }

    debug!("Could not find correct answer");

    // Get the correct answer for gods sake
    Ok(Some(skip(driver)?))
}

pub fn choose_answer_underline_test(
    driver: &WebDriver,
    correct_answer: String,
) -> WebDriverResult<Option<String>> {
    let mut found_all = true;
    debug!("{correct_answer}");
    debug!("{:?}", correct_answer.split(",").collect::<Vec<_>>());
    for part in correct_answer.split(",") {
        debug!("{part}");
        let possibles =
            driver.find_elements(By::Css(r#"div > [data-test="challenge-tap-token"] > span"#))?;

        let mut found = false;

        'inner: for possible in possibles {
            debug!("{}", possible.text()?);
            if possible.text()? == part {
                debug!("Found match");

                possible.click()?;

                found = true;
                break 'inner;
            }
        }

        if !found {
            found_all = false;
            break;
        }
    }

    if !found_all {
        debug!("Could not find correct answer");

        // Get the correct answer for gods sake
        Ok(Some(skip_underline(driver)?))
    } else {
        check_answer_underline(driver)
    }
}

pub fn type_translation(
    driver: &WebDriver,
    correct_answer: String,
) -> WebDriverResult<Option<String>> {
    delay!(100);
    if let Ok(keybd_button) =
        driver.find_element(By::Css(r#"[data-test="player-toggle-keyboard"]"#))
    {
        debug!("{}", keybd_button.text()?);
        let text = keybd_button.text()?.to_lowercase();
        if text == "use keyboard" || text == "make harder" {
            keybd_button.click()?;
        }
    }

    driver
        .find_element(By::Css(
            r#"[data-test="challenge-translate-input"], [data-test="challenge-text-input"]"#,
        ))?
        .send_keys(correct_answer)?;

    delay!(100);

    check_answer_full(&driver)
}
pub fn here_is_tip(driver: &WebDriver) -> WebDriverResult<Option<String>> {
    let potential_question_targets = driver.find_elements(By::Css(r#"div > div > div > span"#))?;

    // Filter for spans with *no attributes*
    let mut pure_spans = vec![];
    for element in potential_question_targets {
        // Jank, I know
        if element.outer_html()?.starts_with("<span>") {
            let element_text = element.text()?;
            pure_spans.push((element, element_text))
        }
    }

    pure_spans.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
    if pure_spans.len() == 0 || pure_spans.len() > 4 {
        // No idea what's happening, try to ignore
        click_next(driver)?;
        return Ok(None);
    }
    let likely_question_text = pure_spans[0].1.clone();

    let possibles = driver.find_elements(By::Css(r#"[data-test="challenge-choice"] > div"#))?;

    for (i, possible) in possibles.iter().enumerate() {
        // Check if possible is contained within question, otherwise just choose last
        if likely_question_text.contains(&possible.text()?) || i == possibles.len() - 1 {
            possible.click()?;
            click_next(driver)?;
            // Check to see if the question was answered correctly
            if check_answer_full(&driver)?.is_some() {
                // Incorrect, so we now have to click next
                click_next(driver)?;
            }
            return Ok(None);
        }
    }
    unreachable!()
}

pub fn type_translation_complete(
    driver: &WebDriver,
    correct_answer: String,
) -> WebDriverResult<Option<String>> {
    delay!(100);
    if let Ok(keybd_button) =
        driver.find_element(By::Css(r#"[data-test="player-toggle-keyboard"]"#))
    {
        let text = keybd_button.text()?.to_lowercase();
        debug!("Switching input: {}?", text);
        if text == "use keyboard" || text == "make harder" {
            debug!("Switched.");
            keybd_button.click()?;
        }
    }

    driver
        .find_element(By::Css(
            r#"[data-test="challenge-translate-input"], [data-test="challenge-text-input"]"#,
        ))?
        .send_keys(correct_answer)?;

    delay!(100);

    check_answer_underline(&driver)
}

/// Click the "check" and checks to see if the answer was correct, if it is incorrect, the full
/// answer is returned
fn check_answer_full(driver: &WebDriver) -> WebDriverResult<Option<String>> {
    click_next(&driver)?;
    delay!(500);

    let result = get_answer_text(&driver)?;

    click_next(&driver)?;

    Ok(result)
}

fn get_answer_text(driver: &WebDriver) -> WebDriverResult<Option<String>> {
    let answer_box = match driver.find_element(By::Css(r#"[data-test="blame blame-incorrect"]"#)) {
        Ok(web_element) => web_element,
        Err(_) => return Ok(None),
    };

    if let Ok(header_elm) = answer_box.find_element(By::Css("h2")) {
        let header_text = header_elm.text()?;
        let answer_text = header_elm
            .find_element(By::XPath("following-sibling::div"))?
            .text()?;

        // Handle case when duolingo gives quadratic answer (multiple options)

        if header_text.to_lowercase().contains("solutions") {
            return Ok(Some(String::from(
                answer_text.split_once(",").unwrap_or((&answer_text, "")).0,
            )));
        } else {
            return Ok(Some(answer_text));
        }
    } else {
        return Ok(None);
    }
}

/// Click the "check" and checks to see if the answer was correct, if it is incorrect, only the
/// underlined component of the answer is returned.
fn check_answer_underline(driver: &WebDriver) -> WebDriverResult<Option<String>> {
    click_next(&driver)?;
    delay!(500);

    let answer_box = match driver.find_element(By::Css(r#"[data-test="blame blame-incorrect"]"#)) {
        Ok(web_element) => web_element,
        Err(_) => {
            click_next(&driver)?;
            return Ok(None);
        }
    };

    let underlined =
        answer_box.find_elements(By::Css(r#"div > div > div > div > span > span[class]"#))?;

    let result = if underlined.is_empty() {
        None
    } else {
        let mut correct = vec![];

        for element in underlined.iter() {
            correct.push(element.text()?);
        }

        Some(correct.join(","))
    };

    click_next(&driver)?;

    Ok(result)
}

pub fn ignore_question(driver: &WebDriver) -> WebDriverResult<()> {
    driver
        .find_element(By::Css(r#"button[data-test="player-skip"]"#))?
        .click()?;

    click_next(driver)?;

    Ok(())
}

pub fn answer_match(
    driver: &WebDriver,
    questions: &HashMap<String, Option<String>>,
) -> WebDriverResult<HashMap<String, String>> {
    let mut response = HashMap::new();
    let mut questions_priority_order: Vec<(&String, &Option<String>)> = questions.iter().collect();
    questions_priority_order.sort_by(|a, b| {
        let a_answered = if a.1.is_none() { 0 } else { 1 };
        let b_answered = if b.1.is_none() { 0 } else { 1 };
        a_answered.cmp(&b_answered)
    });

    for (question, answer) in questions_priority_order {
        if let Some(answer) = answer {
            if !select_pair(driver, question, answer)? {
                response.insert(question.clone(), brute_force(driver, question)?);
            }
        } else {
            response.insert(question.clone(), brute_force(driver, question)?);
        }
    }
    click_next(driver)?;

    Ok(response)
}

fn brute_force(driver: &WebDriver, question: &String) -> WebDriverResult<String> {
    for element in driver.find_elements(By::Css(r#"[data-test="challenge challenge-match"] > div > div > div > div > div:nth-child(2) > div > button"#))? {
        if element.get_attribute("aria-disabled")?.is_none() {
            let text = element.text()?;
            let other_text = element.find_element(By::Tag("span"))?.text()?;
            let answer = text.strip_prefix(other_text.as_str()).unwrap().strip_prefix("\n").unwrap();
            if select_pair(driver, question, answer)? {
                delay!(300);
                return Ok(answer.to_string());
            }
            // Select Pair takes 800ms for animation
            delay!(1000);
        }
    }

    Err(WebDriverError::NoSuchElement(WebDriverErrorInfo::new(
        &format!("Could not find answer for {question}"),
    )))
}

fn select_pair(driver: &WebDriver, question: &str, answer: &str) -> WebDriverResult<bool> {
    select_multi(driver, question, true)?;
    let elm = select_multi(driver, answer, false)?;

    Ok(
        if let Some(disabled) = elm.get_attribute("aria-disabled")? {
            true
        } else {
            false
        },
    )
}

fn select_multi<'a>(
    webdriver: &'a WebDriver,
    text: &str,
    left: bool,
) -> WebDriverResult<WebElement<'a>> {
    let selector = format!(
        r#"[data-test="challenge challenge-match"] > div > div > div > div > div:nth-child({}) > div > button"#,
        if left { 1 } else { 2 }
    );
    for elm in webdriver.find_elements(By::Css(selector.as_str()))? {
        if elm.text()?.ends_with(text) {
            elm.click()?;
            return Ok(elm);
        }
    }

    return Err(WebDriverError::NoSuchElement(WebDriverErrorInfo::new(
        "¯\\_(ツ)_/¯",
    )));
}

pub fn click_on(driver: &WebDriver, data_test: &str) -> WebDriverResult<()> {
    driver
        .find_element(By::Css(&format!(r#"button[data-test="{data_test}"]"#)))?
        .click()
}
