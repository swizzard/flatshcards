use atrium_api::types::TryFromUnknown;
///a stack of flashcards
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Stack {
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    pub back_lang: core::option::Option<String>,
    pub created_at: atrium_api::types::string::Datetime,
    #[serde(skip_serializing_if = "core::option::Option::is_none")]
    pub front_lang: core::option::Option<String>,
    pub label: String,
}
pub type StackRecord = atrium_api::types::Object<Stack>;
impl From<atrium_api::types::Unknown> for Stack {
    fn from(value: atrium_api::types::Unknown) -> Self {
        Self::try_from_unknown(value).unwrap()
    }
}
