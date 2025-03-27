use super::game::{GameEvent, UnoGame};
use super::ui::ConsoleUI;

pub struct GameController {
    game: UnoGame,
    ui: ConsoleUI,
}

impl GameController {
    pub fn new() -> Self {
        let ui = ConsoleUI::new();
        let player_names = ui.get_player_names();
        let game = UnoGame::new(player_names).expect("Failed to start game");
        GameController { game, ui }
    }

    pub fn run(&mut self) {
        println!("Welcome to Uno!");

        loop {
            println!("\n=== Current Turn: Player {} ===", self.game.current_turn);
            self.ui.display_game_state(&self.game);

            // Get current player
            let player = &self.game.players[self.game.current_turn];
            self.ui.display_player_hand(&player.name, &player.hand);

            // Get player action
            let choice = self.ui.get_player_action();

            match choice.as_str() {
                "1" => {
                    // Play a card
                    match self.ui.get_card_index() {
                        Ok(index) => {
                            match self.game.play_card(self.game.current_turn, index) {
                                Ok(event) => {
                                    // Handle Wild and Wild Draw Four color choice
                                    match &event {
                                        GameEvent::WildColorChosen { player_id: _, .. }
                                        | GameEvent::WildDrawFour { player_id: _, .. } => {
                                            let color = self.ui.choose_color();
                                            self.game.discard_pile.last_mut().unwrap().color =
                                                color;
                                            self.ui.handle_game_event(&event, &self.game);
                                        }
                                        _ => self.ui.handle_game_event(&event, &self.game),
                                    }

                                    if let GameEvent::PlayerWins { player_id: _ } = event {
                                        return; // End the game
                                    }
                                }
                                Err(e) => {
                                    println!("Error: {:?}", e);
                                    println!("Please try again.");
                                    continue; // Repeat the turn
                                }
                            }
                        }
                        Err(e) => {
                            println!("{}", e);
                            continue;
                        }
                    }
                }
                "2" => {
                    // Draw a card
                    match self.game.draw_card(self.game.current_turn) {
                        Ok(event) => self.ui.handle_game_event(&event, &self.game),
                        Err(e) => println!("Error: {:?}", e),
                    }
                }
                _ => println!("Invalid choice. Please enter 1 or 2."),
            }

            // Move to the next turn
            self.game.next_turn();
        }
    }
}
