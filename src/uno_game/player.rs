use super::card::Card;
use super::game::GameError;

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(id: usize, name: String) -> Self {
        Self {
            id,
            name,
            hand: Vec::new(),
        }
    }

    /// Adds a card to the player's hand.
    pub fn add_card(&mut self, card: Card) {
        self.hand.push(card);
    }

    /// Removes a card from the player's hand at the specified index.
    /// Returns `Ok(Card)` if the card was successfully removed.
    /// Returns `Err(GameError::CardNotInHand)` if the index is out of bounds.
    pub fn remove_card(&mut self, card_index: usize) -> Result<Card, GameError> {
        if card_index < self.hand.len() {
            Ok(self.hand.remove(card_index))
        } else {
            Err(GameError::CardNotInHand)
        }
    }

    /// Checks if the player has won (i.e., their hand is empty).
    pub fn has_won(&self) -> bool {
        self.hand.is_empty()
    }
}
