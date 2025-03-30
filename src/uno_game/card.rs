use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Wild,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CardType {
    Number(u8),
    Skip,
    Reverse,
    DrawTwo,
    Wild,
    WildDrawFour,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub color: Color,
    pub card_type: CardType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_id: Option<usize>,
}

impl Card {
    pub fn new(color: Color, card_type: CardType) -> Self {
        Self {
            color,
            card_type,
            player_id: None,
        }
    }
}
