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
    pub lang_choices: Vec<(&'static str, &'static str)>,
    pub error: Option<&'a str>,
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
