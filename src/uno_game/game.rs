use super::card::{Card, CardType, Color};
use super::player::Player;
use rand::seq::SliceRandom; // Import the shuffle functionality
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum GameStatus {
    InProgress,
    Complete { winner_id: usize },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UnoGame {
    pub players: Vec<Player>,
    pub deck: Vec<Card>,
    pub discard_pile: Vec<(Card, usize)>, // (Card, player_id) tuples
    pub current_turn: usize,
    pub direction: Direction,
    pub pending_draws: usize, // Number of cards the current player must draw
    pub status: GameStatus,
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
        player_name: String,
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
        let discard_pile = vec![(top_card, usize::MAX)]; // Use usize::MAX to indicate no player played this card

        Ok(Self {
            players,
            deck,
            discard_pile,
            current_turn: 0,
            direction: Direction::Clockwise,
            pending_draws: 0,
            status: GameStatus::InProgress,
        })
    }

    pub fn initialize_deck() -> Vec<Card> {
        let mut deck = Vec::new();

        // Create standard cards
        for &color in &[Color::Red, Color::Green, Color::Blue, Color::Yellow] {
            // Add one copy of the 0 card
            deck.push(Card::new(color.clone(), CardType::Number(0)));

            // Add two copies of each numbered card (1–9)
            for number in 1..=9 {
                deck.push(Card::new(color.clone(), CardType::Number(number)));
                deck.push(Card::new(color.clone(), CardType::Number(number)));
            }

            // Add Skip, Reverse, and Draw Two (two copies each)
            for _ in 0..2 {
                deck.push(Card::new(color.clone(), CardType::Skip));
                deck.push(Card::new(color.clone(), CardType::Reverse));
                deck.push(Card::new(color.clone(), CardType::DrawTwo));
            }
        }

        // Add Wild and Wild Draw Four cards
        for _ in 0..4 {
            deck.push(Card::new(Color::Wild, CardType::Wild));
            deck.push(Card::new(Color::Wild, CardType::WildDrawFour));
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
    pub fn play_card(&mut self, player_id: usize, card_index: usize) -> Result<GameEvent, String> {
        if matches!(self.status, GameStatus::Complete { .. }) {
            return Err("Game is already over".to_string());
        }

        if player_id != self.current_turn {
            return Err("Not your turn".to_string());
        }

        let player = &mut self.players[player_id];
        if card_index >= player.hand.len() {
            return Err("Invalid card index".to_string());
        }

        let card = player.hand.remove(card_index);
        let card_type = card.card_type.clone();
        let player_name = player.name.clone();
        let is_hand_empty = player.hand.is_empty();

        // Add card to discard pile
        self.discard_pile.push((card.clone(), player_id));

        // Check if player has won
        if is_hand_empty {
            self.status = GameStatus::Complete {
                winner_id: player_id,
            };
            return Ok(GameEvent::PlayerWins { player_id });
        }

        // Handle special card effects
        match card_type {
            CardType::Skip => {
                // Skip the next player
                self.next_turn();
                // Move to the player after the skipped one
                self.next_turn();
                Ok(GameEvent::CardPlayed {
                    player_id,
                    player_name,
                    card,
                })
            }
            CardType::Reverse => {
                // Reverse the direction first
                self.reverse_direction();
                // Then move to the next player in the new direction
                self.next_turn();
                Ok(GameEvent::CardPlayed {
                    player_id,
                    player_name,
                    card,
                })
            }
            CardType::DrawTwo => {
                // Set pending draws first
                self.pending_draws = 2;
                // Then move to the next player who must draw
                self.next_turn();
                Ok(GameEvent::CardPlayed {
                    player_id,
                    player_name,
                    card,
                })
            }
            CardType::WildDrawFour => {
                // Set pending draws first
                self.pending_draws = 4;
                // Then move to the next player who must draw
                self.next_turn();
                Ok(GameEvent::CardPlayed {
                    player_id,
                    player_name,
                    card,
                })
            }
            _ => {
                // Normal card - just move to the next player
                self.next_turn();
                Ok(GameEvent::CardPlayed {
                    player_id,
                    player_name,
                    card,
                })
            }
        }
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
            self.next_turn();
            return Ok(GameEvent::DrawTwo { player_id, cards });
        }

        // Normal draw
        if let Some(card) = self.deck.pop() {
            player.hand.push(card.clone());
            self.next_turn();
            Ok(GameEvent::CardDrawn { player_id, card })
        } else {
            Err(GameError::EmptyDeck)
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
        let top_card = &game.discard_pile.last().unwrap().0;

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
            game.discard_pile.last().unwrap().0.card_type,
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
        let red_card = Card::new(Color::Red, CardType::Number(1));
        let blue_card = Card::new(Color::Blue, CardType::Number(1));
        let wild_card = Card::new(Color::Wild, CardType::Wild);

        // Same color
        assert!(UnoGame::can_play_card(&red_card, &red_card));
        // Same number, different color
        assert!(UnoGame::can_play_card(&red_card, &blue_card));
        // Wild card can be played on anything
        assert!(UnoGame::can_play_card(&wild_card, &red_card));
        // Regular card on wild
        assert!(UnoGame::can_play_card(&red_card, &wild_card));
    }

    #[test]
    fn test_wild_draw_four_turn_progression() {
        let player_names = vec!["Martin".to_string(), "Tanya".to_string()];
        let mut game = UnoGame::new(player_names).unwrap();

        // Set up the game state to match the current scenario
        game.current_turn = 1; // Tanya's turn
        game.direction = Direction::Clockwise;

        // Add a WildDrawFour card to Tanya's hand
        let wild_draw_four = Card::new(Color::Wild, CardType::WildDrawFour);
        game.players[1].hand.push(wild_draw_four);

        // Play the WildDrawFour card
        let result = game.play_card(1, 0);
        assert!(result.is_ok());

        // Verify that the turn moved to Martin (player 0)
        assert_eq!(game.current_turn, 0);
        assert_eq!(game.pending_draws, 4);
    }

    #[test]
    fn test_skip_turn_progression() {
        let player_names = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let mut game = UnoGame::new(player_names).unwrap();

        // Set up the game state
        game.current_turn = 0; // Alice's turn
        game.direction = Direction::Clockwise;

        // Add a Skip card to Alice's hand
        let skip_card = Card::new(Color::Red, CardType::Skip);
        game.players[0].hand.push(skip_card);

        // Play the Skip card
        let result = game.play_card(0, 0);
        assert!(result.is_ok());

        // Verify that the turn moved to Charlie (skipping Bob)
        assert_eq!(game.current_turn, 2);
    }

    #[test]
    fn test_reverse_turn_progression() {
        let player_names = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let mut game = UnoGame::new(player_names).unwrap();

        // Set up the game state
        game.current_turn = 1; // Bob's turn
        game.direction = Direction::Clockwise;

        // Add a Reverse card to Bob's hand
        let reverse_card = Card::new(Color::Blue, CardType::Reverse);
        game.players[1].hand.push(reverse_card);

        // Play the Reverse card
        let result = game.play_card(1, 0);
        assert!(result.is_ok());

        // Verify that the direction is reversed and turn moved to Alice
        assert_eq!(game.direction, Direction::CounterClockwise);
        assert_eq!(game.current_turn, 0);
    }

    #[test]
    fn test_draw_two_turn_progression() {
        let player_names = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let mut game = UnoGame::new(player_names).unwrap();

        // Set up the game state
        game.current_turn = 0; // Alice's turn
        game.direction = Direction::Clockwise;

        // Add a Draw Two card to Alice's hand
        let draw_two_card = Card::new(Color::Green, CardType::DrawTwo);
        game.players[0].hand.push(draw_two_card);

        // Play the Draw Two card
        let result = game.play_card(0, 0);
        assert!(result.is_ok());

        // Verify that Bob has pending draws and it's their turn
        assert_eq!(game.pending_draws, 2);
        assert_eq!(game.current_turn, 1);
    }

    #[test]
    fn test_normal_card_turn_progression() {
        let player_names = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let mut game = UnoGame::new(player_names).unwrap();

        // Set up the game state
        game.current_turn = 0; // Alice's turn
        game.direction = Direction::Clockwise;

        // Add a normal card to Alice's hand
        let normal_card = Card::new(Color::Red, CardType::Number(5));
        game.players[0].hand.push(normal_card);

        // Play the normal card
        let result = game.play_card(0, 0);
        assert!(result.is_ok());

        // Verify that the turn moved to Bob
        assert_eq!(game.current_turn, 1);
    }

    #[test]
    fn test_draw_card_turn_progression() {
        let player_names = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let mut game = UnoGame::new(player_names).unwrap();

        // Set up the game state
        game.current_turn = 0; // Alice's turn
        game.direction = Direction::Clockwise;

        // Draw a card
        let result = game.draw_card(0);
        assert!(result.is_ok());

        // Verify that the turn moved to Bob
        assert_eq!(game.current_turn, 1);
    }

    #[test]
    fn test_draw_card_with_pending_draws() {
        let player_names = vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ];
        let mut game = UnoGame::new(player_names).unwrap();

        // Set up the game state
        game.current_turn = 0; // Alice's turn
        game.direction = Direction::Clockwise;
        game.pending_draws = 2; // Alice has pending draws

        // Draw the pending cards
        let result = game.draw_card(0);
        assert!(result.is_ok());

        // Verify that the pending draws are cleared and turn moved to Bob
        assert_eq!(game.pending_draws, 0);
        assert_eq!(game.current_turn, 1);
    }
}
