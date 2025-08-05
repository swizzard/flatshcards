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
    pub fn stack_not_found() -> Self {
        Self {
            title: "Error",
            error: "Stack not found",
        }
    }
    pub fn forbidden() -> Self {
        Self {
            title: "Forbidden",
            error: "You do not have permission to perform this action",
        }
    }
    pub fn db_query() -> Self {
        Self {
            title: "Error",
            error: "Error querying database",
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

#[derive(Template)]
#[template(path = "edit_stack.html")]
pub struct EditStackTemplate<'a> {
    pub title: &'a str,
    pub lang_choices: Vec<(&'a str, &'a str)>,
    pub stack: db::StackDetails,
    pub error: Option<&'a str>,
    pub add_card: AddCardTemplate<'a>,
    pub edit_cards: EditCardsTemplate<'a>,
}

#[derive(Template)]
#[template(path = "add_card.html")]
pub struct AddCardTemplate<'a> {
    pub lang_choices: Vec<(&'a str, &'a str)>,
    pub stack: db::StackDetails,
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "edit_cards.html")]
pub struct EditCardsTemplate<'a> {
    pub lang_choices: Vec<(&'a str, &'a str)>,
    pub cards: Vec<db::DisplayCard>,
    pub stack_id: String,
}

#[derive(Template)]
#[template(path = "form_error.html")]
pub struct FormError<'a> {
    pub error: &'a str,
}
