use clap::Parser;
use std::io::Write;
use std::path::PathBuf;
use uno::uno_game::ui::ConsoleUI;
use uno::uno_game::{start_api_server, GameEvent, GameSession, SessionManager, UnoGame};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run in server mode (API only)
    #[arg(short, long)]
    server: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let sessions_dir = PathBuf::from("sessions");
    let session_manager =
        SessionManager::new(sessions_dir.clone()).expect("Failed to create session manager");

    if args.server {
        // Run in server mode
        println!("Starting Uno API server...");
        if let Err(e) = start_api_server(sessions_dir).await {
            eprintln!("Failed to start API server: {}", e);
            std::process::exit(1);
        }
    } else {
        // Run in CLI mode
        let mut ui = ConsoleUI::new();
        run_cli(&session_manager, &mut ui);
    }
}

fn run_cli(session_manager: &SessionManager, ui: &mut ConsoleUI) {
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
                        play_turn(&mut session, session_manager, ui);
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

fn play_turn(session: &mut GameSession, manager: &SessionManager, ui: &mut ConsoleUI) {
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
            let (index, color) = match ui.get_card_play(&player.hand[0]) {
                Ok(result) => result,
                Err(e) => {
                    println!("{}", e);
                    return;
                }
            };

            if let Some(color) = color {
                session.game.players[session.game.current_turn].hand[index].color = color;
            }

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

    ui.handle_game_event(&event, &session.game);

    if let GameEvent::PlayerWins { player_id: _ } = event {
        println!("Game Over! Player {} wins!", player_name);
        return;
    }

    session.game.next_turn();
    if let Err(e) = session.save(&manager.sessions_dir) {
        println!("Failed to save game state: {}", e);
    }
}
