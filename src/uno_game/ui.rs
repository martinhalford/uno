use super::card::{Card, Color};
use super::game::{GameEvent, UnoGame};
use std::io::{self, Write};

pub struct ConsoleUI;

impl ConsoleUI {
    pub fn new() -> Self {
        ConsoleUI
    }

    pub fn get_player_names(&self) -> Vec<String> {
        let mut player_names = Vec::new();
        loop {
            print!("Enter player name (or '.' to finish): ");
            io::stdout().flush().unwrap();

            let mut name = String::new();
            io::stdin().read_line(&mut name).unwrap();
            let name = name.trim().to_string();

            if name.to_lowercase() == "." {
                if player_names.len() < 2 {
                    println!("You need at least 2 players to start the game.");
                    continue;
                }
                break;
            }

            player_names.push(name);
        }
        player_names
    }

    pub fn display_game_state(&self, game: &UnoGame) {
        println!("\n--- Game State ---");
        println!("Direction: {:?}", game.direction);
        println!(
            "Discard Pile Top Card: {:?}",
            game.discard_pile.last().unwrap()
        );
        println!("Deck Cards Remaining: {}", game.deck.len());
    }

    pub fn display_player_hand(&self, player_name: &str, hand: &[Card]) {
        println!("\nPlayer {}'s hand:", player_name);
        for (i, card) in hand.iter().enumerate() {
            println!("{}. {:?}", i, card);
        }
    }

    pub fn get_player_action(&self) -> String {
        println!("\nWhat would you like to do?");
        println!("1. Play a card");
        println!("2. Draw a card");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        choice.trim().to_string()
    }

    pub fn get_card_index(&self) -> Result<usize, String> {
        print!("Enter the index of the card you want to play: ");
        io::stdout().flush().unwrap();

        let mut index = String::new();
        io::stdin().read_line(&mut index).unwrap();
        index
            .trim()
            .parse::<usize>()
            .map_err(|_| "Invalid input. Please enter a number.".to_string())
    }

    pub fn choose_color(&self) -> Color {
        loop {
            println!("Choose a color:");
            println!("1. Red");
            println!("2. Green");
            println!("3. Blue");
            println!("4. Yellow");
            print!("Enter your choice: ");
            io::stdout().flush().unwrap();

            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            let choice = choice.trim();

            match choice {
                "1" => return Color::Red,
                "2" => return Color::Green,
                "3" => return Color::Blue,
                "4" => return Color::Yellow,
                _ => println!("Invalid choice. Please enter 1, 2, 3, or 4."),
            }
        }
    }

    pub fn handle_game_event(&self, event: &GameEvent, game: &UnoGame) {
        match event {
            GameEvent::CardPlayed { player_id, card } => {
                println!("Player {} played {:?}", game.players[*player_id].name, card);
            }
            GameEvent::CardDrawn { player_id, card } => {
                println!("Player {} drew {:?}", game.players[*player_id].name, card);
            }
            GameEvent::Skip { player_id: _ } => {
                println!("Next player is skipped!");
            }
            GameEvent::Reverse => {
                println!("Direction reversed!");
            }
            GameEvent::DrawTwo { player_id, cards } => {
                println!(
                    "Player {} draws 2 cards: {:?}",
                    game.players[*player_id].name, cards
                );
            }
            GameEvent::WildColorChosen { player_id, color } => {
                println!(
                    "Player {} chose color {:?}",
                    game.players[*player_id].name, color
                );
            }
            GameEvent::WildDrawFour {
                player_id,
                next_player_id,
                cards,
                color,
            } => {
                println!(
                    "Player {} played Wild Draw Four! Player {} draws 4 cards: {:?}",
                    game.players[*player_id].name, game.players[*next_player_id].name, cards
                );
                println!(
                    "Player {} chose color {:?}",
                    game.players[*player_id].name, color
                );
            }
            GameEvent::PlayerWins { player_id } => {
                println!("Player {} has won the game!", game.players[*player_id].name);
            }
        }
    }
}
