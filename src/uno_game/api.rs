use super::card::{Card, CardType, Color};
use super::game::GameStatus;
use crate::uno_game::{GameSession, SessionManager, UnoGame};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};

#[derive(Clone)]
pub struct AppState {
    session_manager: SessionManager,
}

#[derive(Deserialize)]
pub struct CreateGameRequest {
    player_names: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GameResponse {
    id: String,
    current_turn: usize,
    players: Vec<PlayerResponse>,
    discard_pile_top: CardResponse,
    deck_cards_remaining: usize,
    pending_draws: usize,
    status: String,
    winner: Option<WinnerResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerResponse {
    id: usize,
    name: String,
    hand_size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct CardResponse {
    color: String,
    card_type: String,
}

#[derive(Deserialize)]
pub struct PlayCardRequest {
    card_index: usize,
    #[serde(default)]
    color: Option<String>,
}

#[derive(Deserialize)]
pub struct ChooseColorRequest {
    color: String,
}

#[derive(Serialize, Deserialize)]
pub struct GameStateResponse {
    id: String,
    current_turn: usize,
    direction: String,
    players: Vec<PlayerStateResponse>,
    discard_pile_top: CardResponse,
    deck_cards_remaining: usize,
    pending_draws: usize,
    status: String,
    winner: Option<WinnerResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerStateResponse {
    id: usize,
    name: String,
    hand: Vec<(usize, CardResponse)>,
}

#[derive(Serialize, Deserialize)]
pub struct DeckResponse {
    cards: Vec<CardResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct WinnerResponse {
    id: usize,
    name: String,
}

pub async fn create_game(
    State(state): State<AppState>,
    Json(req): Json<CreateGameRequest>,
) -> impl IntoResponse {
    info!("Creating new game with players: {:?}", req.player_names);
    match UnoGame::new(req.player_names) {
        Ok(game) => match state.session_manager.create_session(game) {
            Ok(session) => {
                info!("Created new game session: {}", session.id);
                let response = GameResponse::from_session(&session);
                (StatusCode::CREATED, Json(response)).into_response()
            }
            Err(e) => {
                error!("Failed to create session: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
            }
        },
        Err(e) => {
            error!("Failed to create game: {:?}", e);
            (StatusCode::BAD_REQUEST, e.to_string()).into_response()
        }
    }
}

pub async fn list_games(State(state): State<AppState>) -> impl IntoResponse {
    info!("Listing all games");
    match state.session_manager.list_sessions() {
        Ok(sessions) => {
            info!("Found {} games", sessions.len());
            Json(sessions).into_response()
        }
        Err(e) => {
            error!("Failed to list sessions: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn get_game(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    info!("Getting game with ID: {}", id);
    match state.session_manager.load_session(&id) {
        Ok(session) => {
            info!("Found game: {}", id);
            let response = GameResponse::from_session(&session);
            Json(response).into_response()
        }
        Err(e) => {
            info!("Game not found: {}", id);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

pub async fn get_game_state(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Getting game state for game ID: {}", id);
    match state.session_manager.load_session(&id) {
        Ok(session) => {
            info!("Found game state: {}", id);
            let response = GameStateResponse::from_session(&session);
            Json(response).into_response()
        }
        Err(e) => {
            info!("Game not found: {}", id);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

pub async fn get_deck(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    info!("Getting deck for game ID: {}", id);
    match state.session_manager.load_session(&id) {
        Ok(session) => {
            info!("Found game deck: {}", id);
            let response = DeckResponse {
                cards: session
                    .game
                    .deck
                    .iter()
                    .map(CardResponse::from_card)
                    .collect(),
            };
            Json(response).into_response()
        }
        Err(e) => {
            info!("Game not found: {}", id);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

pub async fn delete_game(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting game with ID: {}", id);
    match state.session_manager.delete_session(&id) {
        Ok(_) => {
            info!("Successfully deleted game: {}", id);
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            info!("Failed to delete game: {} - {}", id, e);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

pub async fn play_card(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PlayCardRequest>,
) -> impl IntoResponse {
    info!("Playing card at index {} in game: {}", req.card_index, id);
    match state.session_manager.load_session(&id) {
        Ok(mut session) => {
            // Check if the card being played is a Wild or Wild Draw Four
            let card = session.game.players[session.game.current_turn]
                .hand
                .get(req.card_index);
            let requires_color = card.map_or(false, |c| {
                matches!(c.card_type, CardType::Wild | CardType::WildDrawFour)
            });

            // If it's a Wild card, require a color
            if requires_color {
                if let Some(color_str) = &req.color {
                    let color = match color_str.to_lowercase().as_str() {
                        "red" => Color::Red,
                        "green" => Color::Green,
                        "blue" => Color::Blue,
                        "yellow" => Color::Yellow,
                        _ => {
                            info!("Invalid color {} in game: {}", color_str, id);
                            return (StatusCode::BAD_REQUEST, "Invalid color".to_string())
                                .into_response();
                        }
                    };
                    // Set the color of the Wild card before playing it
                    session.game.players[session.game.current_turn].hand[req.card_index].color =
                        color;
                } else {
                    info!("Color required for Wild card in game: {}", id);
                    return (
                        StatusCode::BAD_REQUEST,
                        "Color required for Wild card".to_string(),
                    )
                        .into_response();
                }
            }

            match session
                .game
                .play_card(session.game.current_turn, req.card_index)
            {
                Ok(event) => {
                    info!("Successfully played card in game: {}", id);
                    session.game.next_turn();
                    if let Err(e) = session.save(&state.session_manager.sessions_dir) {
                        error!("Failed to save game state: {}", e);
                        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                    }
                    Json(event).into_response()
                }
                Err(e) => {
                    info!("Failed to play card in game: {} - {}", id, e);
                    (StatusCode::BAD_REQUEST, e.to_string()).into_response()
                }
            }
        }
        Err(e) => {
            info!("Game not found: {}", id);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

pub async fn draw_card(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    info!("Drawing card in game: {}", id);
    match state.session_manager.load_session(&id) {
        Ok(mut session) => match session.game.draw_card(session.game.current_turn) {
            Ok(event) => {
                info!("Successfully drew card in game: {}", id);
                session.game.next_turn();
                if let Err(e) = session.save(&state.session_manager.sessions_dir) {
                    error!("Failed to save game state: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
                Json(event).into_response()
            }
            Err(e) => {
                info!("Failed to draw card in game: {} - {}", id, e);
                (StatusCode::BAD_REQUEST, e.to_string()).into_response()
            }
        },
        Err(e) => {
            info!("Game not found: {}", id);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

pub async fn choose_color(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ChooseColorRequest>,
) -> impl IntoResponse {
    info!("Choosing color {} in game: {}", req.color, id);
    // Validate color first
    let color = match req.color.to_lowercase().as_str() {
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "yellow" => Color::Yellow,
        _ => {
            info!("Invalid color {} in game: {}", req.color, id);
            return (StatusCode::BAD_REQUEST, "Invalid color".to_string()).into_response();
        }
    };

    match state.session_manager.load_session(&id) {
        Ok(mut session) => {
            if let Some(top_card) = session.game.discard_pile.last_mut() {
                top_card.color = color;
                if let Err(e) = session.save(&state.session_manager.sessions_dir) {
                    error!("Failed to save game state: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
                info!("Successfully chose color in game: {}", id);
                StatusCode::OK.into_response()
            } else {
                info!("No card in discard pile for game: {}", id);
                (
                    StatusCode::BAD_REQUEST,
                    "No card in discard pile".to_string(),
                )
                    .into_response()
            }
        }
        Err(e) => {
            info!("Game not found: {}", id);
            (StatusCode::NOT_FOUND, e.to_string()).into_response()
        }
    }
}

impl GameResponse {
    fn from_session(session: &GameSession) -> Self {
        let (status, winner) = match session.game.status {
            GameStatus::InProgress => ("In Progress".to_string(), None),
            GameStatus::Complete { winner_id } => (
                "Complete".to_string(),
                Some(WinnerResponse {
                    id: winner_id,
                    name: session.game.players[winner_id].name.clone(),
                }),
            ),
        };

        Self {
            id: session.id.clone(),
            current_turn: session.game.current_turn,
            players: session
                .game
                .players
                .iter()
                .map(|p| PlayerResponse {
                    id: p.id,
                    name: p.name.clone(),
                    hand_size: p.hand.len(),
                })
                .collect(),
            discard_pile_top: CardResponse::from_card(session.game.discard_pile.last().unwrap()),
            deck_cards_remaining: session.game.deck.len(),
            pending_draws: session.game.pending_draws,
            status,
            winner,
        }
    }
}

impl CardResponse {
    fn from_card(card: &Card) -> Self {
        Self {
            color: format!("{:?}", card.color),
            card_type: format!("{:?}", card.card_type),
        }
    }
}

impl GameStateResponse {
    fn from_session(session: &GameSession) -> Self {
        let (status, winner) = match session.game.status {
            GameStatus::InProgress => ("In Progress".to_string(), None),
            GameStatus::Complete { winner_id } => (
                "Complete".to_string(),
                Some(WinnerResponse {
                    id: winner_id,
                    name: session.game.players[winner_id].name.clone(),
                }),
            ),
        };

        Self {
            id: session.id.clone(),
            current_turn: session.game.current_turn,
            direction: format!("{:?}", session.game.direction),
            players: session
                .game
                .players
                .iter()
                .map(|p| PlayerStateResponse {
                    id: p.id,
                    name: p.name.clone(),
                    hand: p
                        .hand
                        .iter()
                        .enumerate()
                        .map(|(i, card)| (i, CardResponse::from_card(card)))
                        .collect(),
                })
                .collect(),
            discard_pile_top: CardResponse::from_card(session.game.discard_pile.last().unwrap()),
            deck_cards_remaining: session.game.deck.len(),
            pending_draws: session.game.pending_draws,
            status,
            winner,
        }
    }
}

pub async fn start_api_server(sessions_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    info!("Starting Uno API server...");

    let session_manager = SessionManager::new(sessions_dir)?;
    let state = AppState { session_manager };

    let cors = CorsLayer::permissive();

    // Create a trace layer for logging
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_response(DefaultOnResponse::new().include_headers(true));

    let app = Router::new()
        .route("/games", post(create_game))
        .route("/games", get(list_games))
        .route("/games/{id}", get(get_game))
        .route("/games/{id}/state", get(get_game_state))
        .route("/games/{id}/deck", get(get_deck))
        .route("/games/{id}", delete(delete_game))
        .route("/games/{id}/play", post(play_card))
        .route("/games/{id}/draw", post(draw_card))
        .route("/games/{id}/color", post(choose_color))
        .layer(cors)
        .layer(trace_layer)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    info!("API server running on http://127.0.0.1:3000");
    info!("Request/response logging enabled");
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
    };
    use serde_json::json;
    use tempfile::tempdir;
    use tower::ServiceExt;

    async fn setup_test_app() -> (Router, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let session_manager = SessionManager::new(temp_dir.path().to_path_buf()).unwrap();
        let state = AppState { session_manager };

        let cors = CorsLayer::permissive();
        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_response(DefaultOnResponse::new().include_headers(true));

        let app = Router::new()
            .route("/games", post(create_game))
            .route("/games", get(list_games))
            .route("/games/{id}", get(get_game))
            .route("/games/{id}/state", get(get_game_state))
            .route("/games/{id}/deck", get(get_deck))
            .route("/games/{id}", delete(delete_game))
            .route("/games/{id}/play", post(play_card))
            .route("/games/{id}/draw", post(draw_card))
            .route("/games/{id}/color", post(choose_color))
            .layer(cors)
            .layer(trace_layer)
            .with_state(state);

        (app, temp_dir)
    }

    #[tokio::test]
    async fn test_create_game() {
        let (app, _temp_dir) = setup_test_app().await;

        let request = Request::builder()
            .method("POST")
            .uri("/games")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "player_names": ["Alice", "Bob"]
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let response: GameResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(response.players.len(), 2);
        assert_eq!(response.players[0].name, "Alice");
        assert_eq!(response.players[1].name, "Bob");
        assert_eq!(response.current_turn, 0);
    }

    #[tokio::test]
    async fn test_list_games() {
        let (app, _temp_dir) = setup_test_app().await;

        // First create a game
        let create_request = Request::builder()
            .method("POST")
            .uri("/games")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "player_names": ["Alice", "Bob"]
                })
                .to_string(),
            ))
            .unwrap();

        app.clone().oneshot(create_request).await.unwrap();

        // Then list games
        let list_request = Request::builder()
            .method("GET")
            .uri("/games")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(list_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let games: Vec<String> = serde_json::from_slice(&body).unwrap();
        assert!(!games.is_empty());
    }

    #[tokio::test]
    async fn test_get_game() {
        let (app, _temp_dir) = setup_test_app().await;

        // First create a game
        let create_request = Request::builder()
            .method("POST")
            .uri("/games")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "player_names": ["Alice", "Bob"]
                })
                .to_string(),
            ))
            .unwrap();

        let create_response = app.clone().oneshot(create_request).await.unwrap();
        let body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: GameResponse = serde_json::from_slice(&body).unwrap();

        // Then get the specific game
        let get_request = Request::builder()
            .method("GET")
            .uri(format!("/games/{}", game.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(get_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let retrieved_game: GameResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(retrieved_game.id, game.id);
    }

    #[tokio::test]
    async fn test_play_card() {
        let (app, _temp_dir) = setup_test_app().await;

        // First create a game
        let create_request = Request::builder()
            .method("POST")
            .uri("/games")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "player_names": ["Alice", "Bob"]
                })
                .to_string(),
            ))
            .unwrap();

        let create_response = app.clone().oneshot(create_request).await.unwrap();
        let body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: GameResponse = serde_json::from_slice(&body).unwrap();

        // Get the current game state
        let get_request = Request::builder()
            .method("GET")
            .uri(format!("/games/{}", game.id))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(get_request).await.unwrap();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let game_state: GameResponse = serde_json::from_slice(&body).unwrap();

        // Verify that the current player has cards
        assert!(
            game_state.players[game_state.current_turn].hand_size > 0,
            "Current player should have cards"
        );

        // If the top card is a Wild Draw Four, we need to choose a color first
        if game_state.discard_pile_top.card_type == "WildDrawFour" {
            let color_request = Request::builder()
                .method("POST")
                .uri(format!("/games/{}/color", game.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "color": "red"
                    })
                    .to_string(),
                ))
                .unwrap();

            let response = app.clone().oneshot(color_request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        // First try to play a card from the player's hand
        let play_request = Request::builder()
            .method("POST")
            .uri(format!("/games/{}/play", game.id))
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "card_index": 0
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.clone().oneshot(play_request).await.unwrap();
        if response.status() == StatusCode::OK {
            // Successfully played a card from hand
            return;
        }

        // If we couldn't play from hand, try drawing cards until we find a playable one
        let mut attempts = 0;
        let max_attempts = 5; // Limit the number of attempts to avoid infinite loops
        while attempts < max_attempts {
            // Draw a card
            let draw_request = Request::builder()
                .method("POST")
                .uri(format!("/games/{}/draw", game.id))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(draw_request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);

            // Get the current game state
            let get_request = Request::builder()
                .method("GET")
                .uri(format!("/games/{}", game.id))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(get_request).await.unwrap();
            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let game_state: GameResponse = serde_json::from_slice(&body).unwrap();

            // Try to play the last card we drew
            let play_request = Request::builder()
                .method("POST")
                .uri(format!("/games/{}/play", game.id))
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({
                        "card_index": game_state.players[game_state.current_turn].hand_size - 1
                    })
                    .to_string(),
                ))
                .unwrap();

            let response = app.clone().oneshot(play_request).await.unwrap();
            if response.status() == StatusCode::OK {
                // Successfully played a card
                return;
            }

            attempts += 1;
        }

        panic!(
            "Failed to find a playable card after {} attempts",
            max_attempts
        );
    }

    #[tokio::test]
    async fn test_draw_card() {
        let (app, _temp_dir) = setup_test_app().await;

        // First create a game
        let create_request = Request::builder()
            .method("POST")
            .uri("/games")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "player_names": ["Alice", "Bob"]
                })
                .to_string(),
            ))
            .unwrap();

        let create_response = app.clone().oneshot(create_request).await.unwrap();
        let body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: GameResponse = serde_json::from_slice(&body).unwrap();

        // Then draw a card
        let draw_request = Request::builder()
            .method("POST")
            .uri(format!("/games/{}/draw", game.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(draw_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_choose_color() {
        let (app, _temp_dir) = setup_test_app().await;

        // First create a game
        let create_request = Request::builder()
            .method("POST")
            .uri("/games")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "player_names": ["Alice", "Bob"]
                })
                .to_string(),
            ))
            .unwrap();

        let create_response = app.clone().oneshot(create_request).await.unwrap();
        let body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: GameResponse = serde_json::from_slice(&body).unwrap();

        // Then choose a color
        let color_request = Request::builder()
            .method("POST")
            .uri(format!("/games/{}/color", game.id))
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "color": "red"
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(color_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_game() {
        let (app, _temp_dir) = setup_test_app().await;

        // First create a game
        let create_request = Request::builder()
            .method("POST")
            .uri("/games")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "player_names": ["Alice", "Bob"]
                })
                .to_string(),
            ))
            .unwrap();

        let create_response = app.clone().oneshot(create_request).await.unwrap();
        let body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: GameResponse = serde_json::from_slice(&body).unwrap();

        // Then delete the game
        let delete_request = Request::builder()
            .method("DELETE")
            .uri(format!("/games/{}", game.id))
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(delete_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // Verify the game is deleted
        let get_request = Request::builder()
            .method("GET")
            .uri(format!("/games/{}", game.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(get_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_error_cases() {
        let (app, _temp_dir) = setup_test_app().await;

        // Test invalid game ID
        let get_request = Request::builder()
            .method("GET")
            .uri("/games/invalid-id")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(get_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Test invalid color
        let color_request = Request::builder()
            .method("POST")
            .uri("/games/some-id/color")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "color": "invalid"
                })
                .to_string(),
            ))
            .unwrap();

        let response = app.oneshot(color_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_game_state() {
        let (app, _temp_dir) = setup_test_app().await;

        // First create a game
        let create_request = Request::builder()
            .method("POST")
            .uri("/games")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "player_names": ["Alice", "Bob"]
                })
                .to_string(),
            ))
            .unwrap();

        let create_response = app.clone().oneshot(create_request).await.unwrap();
        let body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: GameResponse = serde_json::from_slice(&body).unwrap();

        // Then get the game state
        let get_request = Request::builder()
            .method("GET")
            .uri(format!("/games/{}/state", game.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(get_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let game_state: GameStateResponse = serde_json::from_slice(&body).unwrap();

        // Verify the game state
        assert_eq!(game_state.id, game.id);
        assert_eq!(game_state.current_turn, 0);
        assert_eq!(game_state.direction, "Clockwise");
        assert_eq!(game_state.players.len(), 2);
        assert_eq!(game_state.players[0].name, "Alice");
        assert_eq!(game_state.players[1].name, "Bob");
        assert_eq!(game_state.players[0].hand.len(), 7); // Each player starts with 7 cards
        assert_eq!(game_state.players[1].hand.len(), 7);

        // Verify the top card has valid properties
        assert!(!game_state.discard_pile_top.color.is_empty());
        assert!(!game_state.discard_pile_top.card_type.is_empty());

        assert!(game_state.deck_cards_remaining > 0);
    }

    #[tokio::test]
    async fn test_get_deck() {
        let (app, _temp_dir) = setup_test_app().await;

        // First create a game
        let create_request = Request::builder()
            .method("POST")
            .uri("/games")
            .header("Content-Type", "application/json")
            .body(Body::from(
                json!({
                    "player_names": ["Alice", "Bob"]
                })
                .to_string(),
            ))
            .unwrap();

        let create_response = app.clone().oneshot(create_request).await.unwrap();
        let body = to_bytes(create_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: GameResponse = serde_json::from_slice(&body).unwrap();

        // Then get the deck
        let get_request = Request::builder()
            .method("GET")
            .uri(format!("/games/{}/deck", game.id))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(get_request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let deck: DeckResponse = serde_json::from_slice(&body).unwrap();

        // Verify the deck
        assert!(!deck.cards.is_empty(), "Deck should not be empty");
        assert_eq!(
            deck.cards.len(),
            game.deck_cards_remaining,
            "Deck size should match deck_cards_remaining"
        );

        // Verify each card has valid properties
        for card in deck.cards {
            assert!(!card.color.is_empty(), "Card color should not be empty");
            assert!(!card.card_type.is_empty(), "Card type should not be empty");
        }
    }
}
