use std::io::{self, Write};
use uno::uno_game::card::{Card, CardType, Color};
use uno::uno_game::game::{Direction, GameError, GameEvent, UnoGame};

fn main() {
    println!("Welcome to Uno!");

    // Get player names
    let player_names = get_player_names();
    let mut game = match UnoGame::new(player_names) {
        Ok(game) => game,
        Err(e) => {
            println!("Failed to start game: {:?}", e);
            return;
        }
    };

    // Main game loop
    loop {
        println!("\n=== Current Turn: Player {} ===", game.current_turn);
        display_game_state(&game);

        // Get current player
        let player = &game.players[game.current_turn];

        // Display player's hand
        println!("\nPlayer {}'s hand:", player.name);
        for (i, card) in player.hand.iter().enumerate() {
            println!("{}. {:?}", i, card);
        }

        // Prompt for action
        println!("\nWhat would you like to do?");
        println!("1. Play a card");
        println!("2. Draw a card");
        print!("Enter your choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => {
                // Play a card
                print!("Enter the index of the card you want to play: ");
                io::stdout().flush().unwrap();

                let mut index = String::new();
                io::stdin().read_line(&mut index).unwrap();
                let index = match index.trim().parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => {
                        println!("Invalid input. Please enter a number.");
                        continue;
                    }
                };

                match game.play_card(game.current_turn, index) {
                    Ok(event) => {
                        handle_game_event(&event, &game);
                        if let GameEvent::PlayerWins { player_id } = event {
                            println!(
                                "\nPlayer {} has won the game!",
                                game.players[player_id].name
                            );
                            break;
                        }
                    }
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            "2" => {
                // Draw a card
                match game.draw_card(game.current_turn) {
                    Ok(event) => handle_game_event(&event, &game),
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            _ => println!("Invalid choice. Please enter 1 or 2."),
        }

        // Move to the next turn
        game.next_turn();
    }
}

/// Gets player names from the user.
fn get_player_names() -> Vec<String> {
    let mut player_names = Vec::new();
    loop {
        print!("Enter player name (or 'done' to finish): ");
        io::stdout().flush().unwrap();

        let mut name = String::new();
        io::stdin().read_line(&mut name).unwrap();
        let name = name.trim().to_string();

        if name.to_lowercase() == "done" {
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

/// Displays the current game state.
fn display_game_state(game: &UnoGame) {
    println!("\n--- Game State ---");
    println!("Direction: {:?}", game.direction);
    println!(
        "Discard Pile Top Card: {:?}",
        game.discard_pile.last().unwrap()
    );
    println!("Deck Cards Remaining: {}", game.deck.len());
}

/// Handles game events and displays appropriate messages.
fn handle_game_event(event: &GameEvent, game: &UnoGame) {
    match event {
        GameEvent::CardPlayed { player_id, card } => {
            println!("Player {} played {:?}", game.players[*player_id].name, card);
        }
        GameEvent::CardDrawn { player_id, card } => {
            println!("Player {} drew {:?}", game.players[*player_id].name, card);
        }
        GameEvent::Skip { player_id } => {
            println!("Player {} is skipped!", game.players[*player_id].name);
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
        GameEvent::PlayerWins { player_id } => {
            println!("Player {} has won the game!", game.players[*player_id].name);
        }
    }
}
