use super::card::{Card, CardType, Color};
use super::player::Player;
use rand::seq::SliceRandom; // Import the shuffle functionality
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UnoGame {
    pub players: Vec<Player>,
    pub deck: Vec<Card>,
    pub discard_pile: Vec<Card>,
    pub current_turn: usize,
    pub direction: Direction,
    pub pending_draws: usize, // Number of cards the current player must draw
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameError {
    InvalidMove,
    CardNotInHand,
    GameAlreadyOver,
    EmptyDeck,
    Other(String),
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::InvalidMove => write!(f, "Invalid move"),
            GameError::CardNotInHand => write!(f, "Card not in hand"),
            GameError::GameAlreadyOver => write!(f, "Game is already over"),
            GameError::EmptyDeck => write!(f, "Deck is empty"),
            GameError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameEvent {
    CardPlayed {
        player_id: usize,
        card: Card,
    },
    CardDrawn {
        player_id: usize,
        card: Card,
    },
    Skip {
        player_id: usize,
    },
    Reverse,
    DrawTwo {
        player_id: usize,
        cards: Vec<Card>,
    },
    WildColorChosen {
        player_id: usize,
        color: Color,
    },
    WildDrawFour {
        player_id: usize,
        next_player_id: usize,
        cards: Vec<Card>,
        color: Color,
    },
    PlayerWins {
        player_id: usize,
    },
}

/// Represents the direction of play.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}

impl Direction {
    pub fn reverse(&self) -> Self {
        match self {
            Direction::Clockwise => Direction::CounterClockwise,
            Direction::CounterClockwise => Direction::Clockwise,
        }
    }
}

impl UnoGame {
    pub fn new(player_names: Vec<String>) -> Result<Self, GameError> {
        let mut deck = UnoGame::initialize_deck();

        let mut players = player_names
            .into_iter()
            .enumerate()
            .map(|(id, name)| Player::new(id, name))
            .collect::<Vec<_>>();

        // Deal 7 cards to each player
        for _ in 0..7 {
            for player in players.iter_mut() {
                if let Some(card) = deck.pop() {
                    player.add_card(card);
                } else {
                    return Err(GameError::EmptyDeck);
                }
            }
        }

        // Initialize the discard pile
        let top_card = deck.pop().ok_or(GameError::EmptyDeck)?;
        let discard_pile = vec![top_card];

        Ok(Self {
            players,
            deck,
            discard_pile,
            current_turn: 0,
            direction: Direction::Clockwise,
            pending_draws: 0,
        })
    }

    pub fn initialize_deck() -> Vec<Card> {
        let mut deck = Vec::new();

        // Create standard cards
        for &color in &[Color::Red, Color::Green, Color::Blue, Color::Yellow] {
            // Add one copy of the 0 card
            deck.push(Card {
                color: color.clone(),
                card_type: CardType::Number(0),
            });

            // Add two copies of each numbered card (1–9)
            for number in 1..=9 {
                deck.push(Card {
                    color: color.clone(),
                    card_type: CardType::Number(number),
                });
                deck.push(Card {
                    color: color.clone(),
                    card_type: CardType::Number(number),
                });
            }

            // Add Skip, Reverse, and Draw Two (two copies each)
            for _ in 0..2 {
                deck.push(Card {
                    color: color.clone(),
                    card_type: CardType::Skip,
                });
                deck.push(Card {
                    color: color.clone(),
                    card_type: CardType::Reverse,
                });
                deck.push(Card {
                    color: color.clone(),
                    card_type: CardType::DrawTwo,
                });
            }
        }

        // Add Wild and Wild Draw Four cards
        for _ in 0..4 {
            deck.push(Card {
                color: Color::Wild,
                card_type: CardType::Wild,
            });
            deck.push(Card {
                color: Color::Wild,
                card_type: CardType::WildDrawFour,
            });
        }

        // Shuffle the deck
        let mut rng = rand::rng();
        deck.shuffle(&mut rng);

        deck
    }

    /// Updates the current turn based on the direction of play.
    pub fn next_turn(&mut self) {
        let num_players = self.players.len();
        match self.direction {
            Direction::Clockwise => {
                self.current_turn = (self.current_turn + 1) % num_players;
            }
            Direction::CounterClockwise => {
                self.current_turn = (self.current_turn + num_players - 1) % num_players;
            }
        }
    }

    /// Reverses the direction of play.
    pub fn reverse_direction(&mut self) {
        self.direction = self.direction.reverse();
    }

    /// Checks if a card can be played on top of another card.
    pub fn can_play_card(card: &Card, top_card: &Card) -> bool {
        // Wild cards can always be played
        if card.color == Color::Wild
            || matches!(card.card_type, CardType::Wild | CardType::WildDrawFour)
        {
            return true;
        }

        // If the top card is wild, any card can be played on it
        if top_card.color == Color::Wild {
            return true;
        }

        // Same color or same number
        card.color == top_card.color
            || match card.card_type {
                CardType::Number(n) => match top_card.card_type {
                    CardType::Number(m) => n == m,
                    _ => false,
                },
                // Action cards (Skip, Reverse, DrawTwo) can only be played on matching colors
                _ => false,
            }
    }

    /// Handles playing a card.
    pub fn play_card(
        &mut self,
        player_id: usize,
        card_index: usize,
    ) -> Result<GameEvent, GameError> {
        // Check if the player has pending draws
        if self.pending_draws > 0 {
            return Err(GameError::InvalidMove);
        }

        // Check if the card index is valid
        if card_index >= self.players[player_id].hand.len() {
            return Err(GameError::CardNotInHand);
        }

        // Remove the card from the player's hand
        let card = self.players[player_id].hand.remove(card_index);

        // Check if the card can be played
        let top_card = self.discard_pile.last().unwrap();

        if !UnoGame::can_play_card(&card, top_card) {
            // If the card cannot be played, return it to the player's hand
            self.players[player_id].hand.push(card);
            return Err(GameError::InvalidMove);
        }

        // Add the card to the discard pile
        self.discard_pile.push(card.clone());

        // Handle special cards
        let event = self.handle_special_card(player_id, &card)?;

        // Check if the player has won
        if self.players[player_id].hand.is_empty() {
            return Ok(GameEvent::PlayerWins { player_id });
        }

        Ok(event)
    }

    /// Handles drawing a card.
    pub fn draw_card(&mut self, player_id: usize) -> Result<GameEvent, GameError> {
        let player = &mut self.players[player_id];

        // If there are pending draws, draw those cards
        if self.pending_draws > 0 {
            let mut cards = Vec::new();
            for _ in 0..self.pending_draws {
                if let Some(card) = self.deck.pop() {
                    player.hand.push(card.clone());
                    cards.push(card);
                } else {
                    return Err(GameError::EmptyDeck);
                }
            }
            self.pending_draws = 0;
            return Ok(GameEvent::DrawTwo { player_id, cards });
        }

        // Normal draw
        if let Some(card) = self.deck.pop() {
            player.hand.push(card.clone());
            Ok(GameEvent::CardDrawn { player_id, card })
        } else {
            Err(GameError::EmptyDeck)
        }
    }

    /// Handles special card effects.
    fn handle_special_card(
        &mut self,
        player_id: usize,
        card: &Card,
    ) -> Result<GameEvent, GameError> {
        match card.card_type {
            CardType::Skip => {
                self.next_turn(); // Skip the next player
                Ok(GameEvent::Skip {
                    player_id: (self.current_turn + 1) % self.players.len(),
                })
            }
            CardType::Reverse => {
                self.reverse_direction();
                Ok(GameEvent::Reverse)
            }
            CardType::DrawTwo => {
                // Set pending draws for the next player
                self.next_turn();
                self.pending_draws = 2;
                Ok(GameEvent::DrawTwo {
                    player_id: self.current_turn,
                    cards: Vec::new(), // Cards will be drawn when the player draws
                })
            }
            CardType::Wild => {
                Ok(GameEvent::WildColorChosen {
                    player_id,
                    color: card.color, // The color is chosen by the player in the CLI
                })
            }
            CardType::WildDrawFour => {
                // Set pending draws for the next player
                self.next_turn();
                self.pending_draws = 4;
                Ok(GameEvent::WildDrawFour {
                    player_id,
                    next_player_id: self.current_turn,
                    cards: Vec::new(), // Cards will be drawn when the player draws
                    color: card.color, // The color is chosen by the player in the CLI
                })
            }
            _ => Ok(GameEvent::CardPlayed {
                player_id,
                card: card.clone(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_deck() {
        let deck = UnoGame::initialize_deck();
        assert_eq!(deck.len(), 108); // Standard Uno deck has 108 cards
    }

    #[test]
    fn test_new_game() {
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let game = UnoGame::new(player_names).unwrap();

        // Check that each player has 7 cards
        assert_eq!(game.players[0].hand.len(), 7);
        assert_eq!(game.players[1].hand.len(), 7);

        // Check that the discard pile has 1 card
        assert_eq!(game.discard_pile.len(), 1);

        // Check that the deck has the correct number of remaining cards
        assert_eq!(game.deck.len(), 108 - (7 * 2) - 1);

        // Check that the current turn is set to 0 (first player)
        assert_eq!(game.current_turn, 0);

        // Check that the direction is set to Clockwise
        assert_eq!(game.direction, Direction::Clockwise);
    }

    #[test]
    fn test_play_card() {
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let mut game = UnoGame::new(player_names).unwrap();

        // Get the top card of the discard pile
        let top_card = game.discard_pile.last().unwrap();

        // Find a matching card in Alice's hand
        let matching_card_index = game.players[0]
            .hand
            .iter()
            .position(|card| UnoGame::can_play_card(card, top_card))
            .unwrap();

        // Store the card we're going to play
        let card_to_play = game.players[0].hand[matching_card_index].clone();

        // Play the matching card
        let result = game.play_card(0, matching_card_index);
        assert!(result.is_ok());

        // Check that the card was moved to the discard pile
        assert_eq!(
            game.discard_pile.last().unwrap().card_type,
            card_to_play.card_type
        );
    }

    #[test]
    fn test_draw_card() {
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let mut game = UnoGame::new(player_names).unwrap();

        let initial_hand_size = game.players[0].hand.len();
        let initial_deck_size = game.deck.len();

        let result = game.draw_card(0);
        assert!(result.is_ok());

        // Check that the player got a new card
        assert_eq!(game.players[0].hand.len(), initial_hand_size + 1);
        // Check that the deck lost a card
        assert_eq!(game.deck.len(), initial_deck_size - 1);
    }

    #[test]
    fn test_next_turn() {
        let player_names = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let mut game = UnoGame::new(player_names).unwrap();

        // Initial turn is 0 (Alice)
        assert_eq!(game.current_turn, 0);

        // Move to the next turn (Bob)
        game.next_turn();
        assert_eq!(game.current_turn, 1);

        // Move to the next turn (Charlie)
        game.next_turn();
        assert_eq!(game.current_turn, 2);

        // Move to the next turn (Alice, wraps around)
        game.next_turn();
        assert_eq!(game.current_turn, 0);
    }

    #[test]
    fn test_reverse_direction() {
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let mut game = UnoGame::new(player_names).unwrap();

        // Initial direction is Clockwise
        assert_eq!(game.direction, Direction::Clockwise);

        // Reverse the direction
        game.reverse_direction();
        assert_eq!(game.direction, Direction::CounterClockwise);

        // Reverse the direction again
        game.reverse_direction();
        assert_eq!(game.direction, Direction::Clockwise);
    }

    #[test]
    fn test_next_turn_reversed() {
        let player_names = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let mut game = UnoGame::new(player_names).unwrap();

        // Reverse the direction
        game.reverse_direction();
        assert_eq!(game.direction, Direction::CounterClockwise);

        // Initial turn is 0 (Alice)
        assert_eq!(game.current_turn, 0);

        // Move to the next turn (Charlie, since direction is reversed)
        game.next_turn();
        assert_eq!(game.current_turn, 2);

        // Move to the next turn (Bob)
        game.next_turn();
        assert_eq!(game.current_turn, 1);

        // Move to the next turn (Alice, wraps around)
        game.next_turn();
        assert_eq!(game.current_turn, 0);
    }

    #[test]
    fn test_can_play_card() {
        let red_card = Card {
            color: Color::Red,
            card_type: CardType::Number(1),
        };
        let blue_card = Card {
            color: Color::Blue,
            card_type: CardType::Number(1),
        };
        let wild_card = Card {
            color: Color::Wild,
            card_type: CardType::Wild,
        };

        // Same color
        assert!(UnoGame::can_play_card(&red_card, &red_card));
        // Same number, different color
        assert!(UnoGame::can_play_card(&red_card, &blue_card));
        // Wild card can be played on anything
        assert!(UnoGame::can_play_card(&wild_card, &red_card));
        // Regular card on wild
        assert!(UnoGame::can_play_card(&red_card, &wild_card));
    }
}
