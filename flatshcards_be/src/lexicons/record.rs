use super::xyz::flatshcards::{card, stack};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "$type")]
pub enum KnownRecord {
    #[serde(rename = "xyz.flatshcards.cards#stack")]
    LexiconXyzFlatshcardsCardsStack(Box<stack::StackRecord>),
    #[serde(rename = "xyz.flatshcards.cards#card")]
    LexiconXyzFlatshcardsCardsCard(Box<card::CardRecord>),
}

impl From<stack::StackRecord> for KnownRecord {
    fn from(record: stack::StackRecord) -> Self {
        KnownRecord::LexiconXyzFlatshcardsCardsStack(Box::new(record))
    }
}
impl From<stack::Stack> for KnownRecord {
    fn from(record: stack::Stack) -> Self {
        KnownRecord::LexiconXyzFlatshcardsCardsStack(Box::new(record.into()))
    }
}
impl From<card::CardRecord> for KnownRecord {
    fn from(record: card::CardRecord) -> Self {
        KnownRecord::LexiconXyzFlatshcardsCardsCard(Box::new(record))
    }
}
impl From<card::Card> for KnownRecord {
    fn from(record: card::Card) -> Self {
        KnownRecord::LexiconXyzFlatshcardsCardsCard(Box::new(record.into()))
    }
}
#[allow(clippy::from_over_into)]
impl Into<atrium_api::types::Unknown> for KnownRecord {
    fn into(self) -> atrium_api::types::Unknown {
        atrium_api::types::TryIntoUnknown::try_into_unknown(&self).unwrap()
    }
}
