#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Red,
    Green,
    Blue,
    Yellow,
    Wild,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardType {
    Number(u8),
    Skip,
    Reverse,
    DrawTwo,
    Wild,
    WildDrawFour,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Card {
    pub color: Color,
    pub card_type: CardType,
}
