use atrium_api::types::TryFromUnknown;
///a single flashcard
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub back_lang: String,
    pub back_text: String,
    pub created_at: atrium_api::types::string::Datetime,
    pub front_lang: String,
    pub front_text: String,
    pub stack_id: atrium_api::types::string::RecordKey,
}
pub type CardRecord = atrium_api::types::Object<Card>;
impl From<atrium_api::types::Unknown> for Card {
    fn from(value: atrium_api::types::Unknown) -> Self {
        Self::try_from_unknown(value).unwrap()
    }
}
