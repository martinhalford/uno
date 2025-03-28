use std::io::{self, Write};
use std::path::PathBuf;
use uno::uno_game::ui::ConsoleUI;
use uno::uno_game::{GameEvent, GameSession, SessionManager, UnoGame};

fn main() {
    let sessions_dir = PathBuf::from("sessions");
    let session_manager =
        SessionManager::new(sessions_dir).expect("Failed to create session manager");
    let ui = ConsoleUI::new();

    loop {
        println!("\nUno Game Session Manager");
        println!("1. Create new game");
        println!("2. List games");
        println!("3. Load game");
        println!("4. Delete game");
        println!("5. Exit");
        print!("Enter your choice: ");
        std::io::stdout().flush().unwrap();

        let mut choice = String::new();
        std::io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => {
                let player_names = ui.get_player_names();
                match UnoGame::new(player_names) {
                    Ok(game) => match session_manager.create_session(game) {
                        Ok(session) => println!("Created new game session: {}", session.id),
                        Err(e) => println!("Failed to create session: {}", e),
                    },
                    Err(e) => println!("Failed to create game: {:?}", e),
                }
            }
            "2" => match session_manager.list_sessions() {
                Ok(sessions) => {
                    if sessions.is_empty() {
                        println!("No active games");
                    } else {
                        println!("Active games:");
                        for id in sessions {
                            println!("- {}", id);
                        }
                    }
                }
                Err(e) => println!("Failed to list sessions: {}", e),
            },
            "3" => {
                print!("Enter game ID: ");
                std::io::stdout().flush().unwrap();
                let mut id = String::new();
                std::io::stdin().read_line(&mut id).unwrap();
                let id = id.trim();

                match session_manager.load_session(id) {
                    Ok(mut session) => {
                        println!("Loaded game session: {}", session.id);
                        play_turn(&mut session, &session_manager, &ui);
                    }
                    Err(e) => println!("Failed to load session: {}", e),
                }
            }
            "4" => {
                print!("Enter game ID to delete: ");
                std::io::stdout().flush().unwrap();
                let mut id = String::new();
                std::io::stdin().read_line(&mut id).unwrap();
                let id = id.trim();

                match session_manager.delete_session(id) {
                    Ok(_) => println!("Deleted game session: {}", id),
                    Err(e) => println!("Failed to delete session: {}", e),
                }
            }
            "5" => break,
            _ => println!("Invalid choice. Please enter 1-5."),
        }
    }
}

fn play_turn(session: &mut GameSession, manager: &SessionManager, ui: &ConsoleUI) {
    println!(
        "\n=== Current Turn: Player {} ===",
        session.game.current_turn
    );
    ui.display_game_state(&session.game);

    let player_name = session.game.players[session.game.current_turn].name.clone();
    let player = &session.game.players[session.game.current_turn];
    ui.display_player_hand(&player.name, &player.hand);

    let choice = ui.get_player_action();
    let event = match choice.as_str() {
        "1" => {
            let index = match ui.get_card_index() {
                Ok(index) => index,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            };

            match session.game.play_card(session.game.current_turn, index) {
                Ok(event) => event,
                Err(e) => {
                    println!("Error: {:?}", e);
                    return;
                }
            }
        }
        "2" => match session.game.draw_card(session.game.current_turn) {
            Ok(event) => event,
            Err(e) => {
                println!("Error: {:?}", e);
                return;
            }
        },
        _ => {
            println!("Invalid choice. Please enter 1 or 2.");
            return;
        }
    };

    match &event {
        GameEvent::WildColorChosen { player_id: _, .. }
        | GameEvent::WildDrawFour { player_id: _, .. } => {
            let color = ui.choose_color();
            session.game.discard_pile.last_mut().unwrap().color = color;
            ui.handle_game_event(&event, &session.game);
        }
        _ => ui.handle_game_event(&event, &session.game),
    }

    if let GameEvent::PlayerWins { player_id: _ } = event {
        println!("Game Over! Player {} wins!", player_name);
        return;
    }

    session.game.next_turn();
    if let Err(e) = session.save(&manager.sessions_dir) {
        println!("Failed to save game state: {}", e);
    }
}
