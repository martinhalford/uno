use super::card::{Card, CardType, Color};
use super::game::{GameEvent, UnoGame};
use std::io::{self, BufRead, BufReader, Write};

pub struct ConsoleUI {
    input: Box<dyn BufRead>,
    output: Box<dyn Write>,
}

impl ConsoleUI {
    pub fn new() -> Self {
        Self {
            input: Box::new(BufReader::new(io::stdin())),
            output: Box::new(io::stdout()),
        }
    }

    pub fn with_streams(input: Box<dyn BufRead>, output: Box<dyn Write>) -> Self {
        Self { input, output }
    }

    pub fn get_player_names(&mut self) -> Vec<String> {
        let mut player_names = Vec::new();
        loop {
            write!(self.output, "Enter player name (or '.' to finish): ").unwrap();
            self.output.flush().unwrap();

            let mut name = String::new();
            self.input.read_line(&mut name).unwrap();
            let name = name.trim().to_string();

            if name.to_lowercase() == "." {
                if player_names.len() < 2 {
                    writeln!(
                        self.output,
                        "You need at least 2 players to start the game."
                    )
                    .unwrap();
                    continue;
                }
                break;
            }

            player_names.push(name);
        }
        player_names
    }

    pub fn display_game_state(&mut self, game: &UnoGame) {
        writeln!(self.output, "\n--- Game State ---").unwrap();
        writeln!(self.output, "Direction: {:?}", game.direction).unwrap();
        writeln!(
            self.output,
            "Discard Pile Top Card: {:?}",
            game.discard_pile.last().unwrap()
        )
        .unwrap();
        writeln!(self.output, "Deck Cards Remaining: {}", game.deck.len()).unwrap();

        // Show pending draws if any
        if game.pending_draws > 0 {
            writeln!(
                self.output,
                "⚠️  Current player must draw {} cards!",
                game.pending_draws
            )
            .unwrap();
        }
    }

    pub fn display_player_hand(&mut self, player_name: &str, hand: &[Card]) {
        writeln!(self.output, "\nPlayer {}'s hand:", player_name).unwrap();
        for (i, card) in hand.iter().enumerate() {
            writeln!(self.output, "{}. {:?}", i, card).unwrap();
        }
    }

    pub fn get_player_action(&mut self) -> String {
        writeln!(self.output, "\nWhat would you like to do?").unwrap();
        writeln!(self.output, "1. Play a card").unwrap();
        writeln!(self.output, "2. Draw a card").unwrap();
        write!(self.output, "Enter your choice: ").unwrap();
        self.output.flush().unwrap();

        let mut choice = String::new();
        self.input.read_line(&mut choice).unwrap();
        choice.trim().to_string()
    }

    pub fn get_card_index(&mut self) -> Result<(usize, Option<Color>), String> {
        write!(
            self.output,
            "Enter the index of the card you want to play: "
        )
        .unwrap();
        self.output.flush().unwrap();

        let mut index = String::new();
        self.input.read_line(&mut index).unwrap();
        let index = index
            .trim()
            .parse::<usize>()
            .map_err(|_| "Invalid input. Please enter a number.".to_string())?;

        Ok((index, None))
    }

    pub fn get_card_play(&mut self, card: &Card) -> Result<(usize, Option<Color>), String> {
        write!(
            self.output,
            "Enter the index of the card you want to play: "
        )
        .unwrap();
        self.output.flush().unwrap();

        let mut index = String::new();
        self.input.read_line(&mut index).unwrap();
        let index = index
            .trim()
            .parse::<usize>()
            .map_err(|_| "Invalid input. Please enter a number.".to_string())?;

        // If the card is a Wild or Wild Draw Four, get the color choice
        if matches!(card.card_type, CardType::Wild | CardType::WildDrawFour) {
            let color = self.choose_color();
            Ok((index, Some(color)))
        } else {
            Ok((index, None))
        }
    }

    pub fn choose_color(&mut self) -> Color {
        loop {
            writeln!(self.output, "Choose a color:").unwrap();
            writeln!(self.output, "1. Red").unwrap();
            writeln!(self.output, "2. Green").unwrap();
            writeln!(self.output, "3. Blue").unwrap();
            writeln!(self.output, "4. Yellow").unwrap();
            write!(self.output, "Enter your choice: ").unwrap();
            self.output.flush().unwrap();

            let mut choice = String::new();
            self.input.read_line(&mut choice).unwrap();
            let choice = choice.trim();

            match choice {
                "1" => return Color::Red,
                "2" => return Color::Green,
                "3" => return Color::Blue,
                "4" => return Color::Yellow,
                _ => writeln!(self.output, "Invalid choice. Please enter 1, 2, 3, or 4.").unwrap(),
            }
        }
    }

    pub fn handle_game_event(&mut self, event: &GameEvent, game: &UnoGame) {
        match event {
            GameEvent::CardPlayed {
                player_id: _,
                card,
                player_name,
            } => {
                writeln!(self.output, "Player {} played {:?}", player_name, card).unwrap();
            }
            GameEvent::CardDrawn { player_id, card } => {
                writeln!(
                    self.output,
                    "Player {} drew {:?}",
                    game.players[*player_id].name, card
                )
                .unwrap();
            }
            GameEvent::Skip { player_id: _ } => {
                writeln!(self.output, "Next player is skipped!").unwrap();
            }
            GameEvent::Reverse => {
                writeln!(self.output, "Direction reversed!").unwrap();
            }
            GameEvent::DrawTwo { player_id, cards } => {
                writeln!(
                    self.output,
                    "Player {} draws 2 cards: {:?}",
                    game.players[*player_id].name, cards
                )
                .unwrap();
            }
            GameEvent::WildColorChosen { player_id, color } => {
                writeln!(
                    self.output,
                    "Player {} chose color {:?}",
                    game.players[*player_id].name, color
                )
                .unwrap();
            }
            GameEvent::WildDrawFour {
                player_id,
                next_player_id,
                cards,
                color,
            } => {
                writeln!(
                    self.output,
                    "Player {} played Wild Draw Four! Player {} draws 4 cards: {:?}",
                    game.players[*player_id].name, game.players[*next_player_id].name, cards
                )
                .unwrap();
                writeln!(
                    self.output,
                    "Player {} chose color {:?}",
                    game.players[*player_id].name, color
                )
                .unwrap();
            }
            GameEvent::PlayerWins { player_id } => {
                writeln!(
                    self.output,
                    "Player {} has won the game!",
                    game.players[*player_id].name
                )
                .unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uno_game::card::CardType;
    use crate::uno_game::player::Player;
    use std::io::Cursor;

    fn create_test_ui() -> ConsoleUI {
        ConsoleUI::new()
    }

    #[test]
    fn test_display_game_state() {
        let mut ui = create_test_ui();
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let game = UnoGame::new(player_names).unwrap();

        // This test just verifies that display_game_state doesn't panic
        ui.display_game_state(&game);
    }

    #[test]
    fn test_display_player_hand() {
        let mut ui = create_test_ui();
        let player = Player::new(0, "Alice".to_string());

        // This test just verifies that display_player_hand doesn't panic
        ui.display_player_hand(&player.name, &player.hand);
    }

    #[test]
    fn test_get_card_index() {
        let input = Cursor::new("5\n");
        let output = Vec::new();
        let mut ui = ConsoleUI::with_streams(Box::new(input), Box::new(output));

        let result = ui.get_card_index();
        assert!(result.is_ok());

        let input = Cursor::new("invalid\n");
        let output = Vec::new();
        let mut ui = ConsoleUI::with_streams(Box::new(input), Box::new(output));
        let result = ui.get_card_index();
        assert!(result.is_err());
    }

    #[test]
    fn test_choose_color() {
        let input = Cursor::new("1\n2\n3\n4\n");
        let output = Vec::new();
        let mut ui = ConsoleUI::with_streams(Box::new(input), Box::new(output));

        // Test valid color choices
        let colors = vec![
            ("1", Color::Red),
            ("2", Color::Green),
            ("3", Color::Blue),
            ("4", Color::Yellow),
        ];

        for (_, expected) in colors {
            let color = ui.choose_color();
            assert_eq!(color, expected);
        }
    }

    #[test]
    fn test_handle_game_event() {
        let mut ui = create_test_ui();
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let game = UnoGame::new(player_names).unwrap();

        // Test various game events
        let events = vec![
            GameEvent::CardPlayed {
                player_id: 0,
                card: Card::new(Color::Red, CardType::Number(1)),
                player_name: "Alice".to_string(),
            },
            GameEvent::Skip { player_id: 1 },
            GameEvent::Reverse,
            GameEvent::DrawTwo {
                player_id: 0,
                cards: vec![Card::new(Color::Blue, CardType::Number(2))],
            },
            GameEvent::WildColorChosen {
                player_id: 0,
                color: Color::Red,
            },
            GameEvent::WildDrawFour {
                player_id: 0,
                next_player_id: 1,
                cards: vec![Card::new(Color::Wild, CardType::WildDrawFour)],
                color: Color::Red,
            },
            GameEvent::PlayerWins { player_id: 0 },
        ];

        // Verify that handle_game_event doesn't panic for any event type
        for event in events {
            ui.handle_game_event(&event, &game);
        }
    }
}
