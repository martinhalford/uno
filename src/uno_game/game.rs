use super::card::{Card, CardType, Color};
use super::player::Player;
use rand::seq::SliceRandom; // Import the shuffle functionality
use rand::thread_rng; // For random number generation

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

    // Other methods like play_card, draw_card, etc.
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
