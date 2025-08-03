///The askama template types for HTML
///
use crate::db;
use askama::Template;
use serde::{Deserialize, Serialize};

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate<'a> {
    pub title: &'a str,
    pub profile: Option<Profile>,
    pub stacks: Vec<db::StackDetails>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub did: String,
    pub display_name: Option<String>,
}

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    pub title: &'a str,
    pub error: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate<'a> {
    pub title: &'a str,
    pub error: &'a str,
}

impl<'a> ErrorTemplate<'a> {
    pub fn session_agent_did() -> Self {
        Self {
            title: "Error",
            error: "Error retrieving AtProto agent",
        }
    }
}

#[derive(Template)]
#[template(path = "create_stack.html")]
pub struct CreateStackTemplate<'a> {
    pub title: &'a str,
    pub lang_choices: Vec<(&'a str, &'a str)>,
    pub error: Option<&'a str>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EditStack<'a> {
    pub uri: &'a str,
    pub label: &'a str,
    pub front_lang: &'a str,
    pub back_lang: &'a str,
}

#[derive(Template)]
#[template(path = "edit_stack.html")]
pub struct EditStackTemplate<'a> {
    pub title: &'a str,
    pub lang_choices: Vec<(&'a str, &'a str)>,
    pub stack: EditStack<'a>,
    pub error: Option<&'a str>,
}
