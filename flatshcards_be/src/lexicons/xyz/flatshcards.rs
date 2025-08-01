use atrium_api::types::Collection;
pub mod card;
pub mod stack;

#[derive(Debug)]
pub struct Stack;
impl Collection for Stack {
    const NSID: &'static str = "xyz.flatshcards.stack";
    type Record = stack::StackRecord;
}

#[derive(Debug)]
pub struct Card;
impl Collection for Card {
    const NSID: &'static str = "xyz.flatshcards.card";
    type Record = card::CardRecord;
}
