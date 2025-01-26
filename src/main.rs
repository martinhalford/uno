use std::io::{self, Write};
use uno::uno_game::card::{Card, CardType, Color};
use uno::uno_game::game::{Direction, GameError, UnoGame};

fn main() {
    // Welcome message
    println!("Welcome to the UNO Game!");

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

        println!("Choice: {}", choice);

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

                match play_card(&mut game, index) {
                    Ok(_) => {
                        // Check if the player has won
                        if game.players[game.current_turn].hand.is_empty() {
                            println!(
                                "\nPlayer {} has won the game!",
                                game.players[game.current_turn].name
                            );
                            break;
                        }
                    }
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            "2" => {
                // Draw a card
                match draw_card(&mut game) {
                    Ok(_) => println!("You drew a card."),
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

/// Handles playing a card.
fn play_card(game: &mut UnoGame, index: usize) -> Result<(), GameError> {
    let player = &mut game.players[game.current_turn];

    // Check if the card index is valid
    if index >= player.hand.len() {
        return Err(GameError::CardNotInHand);
    }

    let card = player.hand[index].clone();

    // Check if the card can be played
    let top_card = game.discard_pile.last().unwrap();
    if !can_play_card(&card, top_card) {
        return Err(GameError::InvalidMove);
    }

    // Remove the card from the player's hand and add it to the discard pile
    player.hand.remove(index);
    game.discard_pile.push(card);

    // Handle special cards
    handle_special_card(game, &card);

    Ok(())
}

fn can_play_card(card: &Card, top_card: &Card) -> bool {
    card.color == top_card.color
        || match card.card_type {
            CardType::Number(n) => match top_card.card_type {
                CardType::Number(m) => n == m,
                _ => false,
            },
            _ => true, // Wild cards can always be played
        }
}

/// Handles special card effects.
fn handle_special_card(game: &mut UnoGame, card: &Card) {
    match card.card_type {
        CardType::Skip => {
            println!(
                "Player {} is skipped!",
                game.players[game.current_turn].name
            );
            game.next_turn(); // Skip the next player
        }
        CardType::Reverse => {
            println!("Direction reversed!");
            game.reverse_direction();
        }
        CardType::DrawTwo => {
            println!("Next player draws 2 cards!");
            let next_player = (game.current_turn + 1) % game.players.len();
            for _ in 0..2 {
                if let Some(card) = game.deck.pop() {
                    game.players[next_player].hand.push(card);
                }
            }
        }
        CardType::Wild | CardType::WildDrawFour => {
            println!(
                "Player {} chooses a color!",
                game.players[game.current_turn].name
            );
            let color = choose_color();
            game.discard_pile.last_mut().unwrap().color = color;
        }
        _ => {}
    }
}

/// Prompts the player to choose a color for a Wild card.
fn choose_color() -> Color {
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

/// Handles drawing a card.
fn draw_card(game: &mut UnoGame) -> Result<(), GameError> {
    let player = &mut game.players[game.current_turn];

    if let Some(card) = game.deck.pop() {
        player.hand.push(card);
        Ok(())
    } else {
        Err(GameError::EmptyDeck)
    }
}
