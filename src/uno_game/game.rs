use super::card::{Card, CardType, Color};
use super::player::Player;
use rand::seq::SliceRandom; // Import the shuffle functionality
use rand::thread_rng; // For random number generation

#[derive(Debug, PartialEq)]
pub struct UnoGame {
    pub players: Vec<Player>,
    pub deck: Vec<Card>,
    pub discard_pile: Vec<Card>,
    pub current_turn: usize,
    pub direction: Direction,
}

#[derive(Debug)]
pub enum GameError {
    InvalidMove,
    CardNotInHand,
    GameAlreadyOver,
    EmptyDeck,
    Other(String),
}

#[derive(Debug)]
pub enum GameEvent {
    CardPlayed { player_id: usize, card: Card },
    CardDrawn { player_id: usize, card: Card },
    Skip { player_id: usize },
    Reverse,
    DrawTwo { player_id: usize, cards: Vec<Card> },
    WildColorChosen { player_id: usize, color: Color },
    PlayerWins { player_id: usize },
}

/// Represents the direction of play.
#[derive(Debug, Clone, Copy, PartialEq)]
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
        let mut rng = thread_rng();
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
        card.color == top_card.color
            || match card.card_type {
                CardType::Number(n) => match top_card.card_type {
                    CardType::Number(m) => n == m,
                    _ => false,
                },
                _ => true, // Wild cards can always be played
            }
    }

    /// Handles playing a card.
    pub fn play_card(
        &mut self,
        player_id: usize,
        card_index: usize,
    ) -> Result<GameEvent, GameError> {
        #[cfg(debug_assertions)]
        eprintln!(
            "[DEBUG] Player {}'s hand before playing: {:?}",
            self.players[player_id].name, self.players[player_id].hand
        );

        // Check if the card index is valid
        if card_index >= self.players[player_id].hand.len() {
            return Err(GameError::CardNotInHand);
        }

        // Remove the card from the player's hand
        let card = self.players[player_id].hand.remove(card_index);

        // Debug output: Print the card being played
        #[cfg(debug_assertions)]
        eprintln!(
            "[DEBUG] Player {} is playing: {:?}",
            self.players[player_id].name, card
        );

        // Check if the card can be played
        let top_card = self.discard_pile.last().unwrap();

        if !UnoGame::can_play_card(&card, top_card) {
            // If the card cannot be played, return it to the player's hand
            self.players[player_id].hand.push(card);
            return Err(GameError::InvalidMove);
        }

        // Add the card to the discard pile
        self.discard_pile.push(card.clone());

        // Debug output: Print the discard pile after playing the card
        #[cfg(debug_assertions)]
        eprintln!(
            "[DEBUG] Discard pile after playing: {:?}",
            self.discard_pile
        );

        // Handle special cards
        let event = self.handle_special_card(player_id, &card)?;

        // Check if the player has won
        if self.players[player_id].hand.is_empty() {
            return Ok(GameEvent::PlayerWins { player_id });
        }

        Ok(event)
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
                let next_player = (self.current_turn + 1) % self.players.len();
                let mut cards = Vec::new();
                for _ in 0..2 {
                    if let Some(card) = self.deck.pop() {
                        self.players[next_player].hand.push(card.clone());
                        cards.push(card);
                    }
                }
                Ok(GameEvent::DrawTwo {
                    player_id: next_player,
                    cards,
                })
            }
            CardType::Wild | CardType::WildDrawFour => {
                Ok(GameEvent::WildColorChosen {
                    player_id,
                    color: card.color, // The color is chosen by the player in the CLI
                })
            }
            _ => Ok(GameEvent::CardPlayed {
                player_id,
                card: card.clone(),
            }),
        }
    }

    /// Handles drawing a card.
    pub fn draw_card(&mut self, player_id: usize) -> Result<GameEvent, GameError> {
        let player = &mut self.players[player_id];

        if let Some(card) = self.deck.pop() {
            player.hand.push(card.clone());
            Ok(GameEvent::CardDrawn { player_id, card })
        } else {
            Err(GameError::EmptyDeck)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the deck is initialized with the correct number of cards.
    #[test]
    fn test_initialize_deck() {
        let deck = UnoGame::initialize_deck();
        assert_eq!(deck.len(), 108); // Standard Uno deck has 108 cards
    }

    /// Tests that a new game is initialized correctly.
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

    /// Tests that the game handles an empty deck during initialization.
    #[test]
    fn test_new_game_empty_deck() {
        let player_names = vec!["Alice".to_string(); 20]; // Too many players to deal 7 cards each
        let result = UnoGame::new(player_names);
        assert!(matches!(result, Err(GameError::EmptyDeck)));
    }

    /// Tests that the next_turn method updates the current turn correctly.
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

    /// Tests that the reverse_direction method toggles the direction correctly.
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

    /// Tests that the next_turn method works correctly when the direction is reversed.
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
}
