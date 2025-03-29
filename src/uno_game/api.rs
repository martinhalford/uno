use super::card::{Card, Color};
use super::{GameSession, SessionManager, UnoGame};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    session_manager: SessionManager,
}

#[derive(Deserialize)]
pub struct CreateGameRequest {
    player_names: Vec<String>,
}

#[derive(Serialize)]
pub struct GameResponse {
    id: String,
    current_turn: usize,
    players: Vec<PlayerResponse>,
    discard_pile_top: CardResponse,
    deck_cards_remaining: usize,
}

#[derive(Serialize)]
pub struct PlayerResponse {
    id: usize,
    name: String,
    hand_size: usize,
}

#[derive(Serialize)]
pub struct CardResponse {
    color: String,
    card_type: String,
}

#[derive(Deserialize)]
pub struct PlayCardRequest {
    card_index: usize,
}

#[derive(Deserialize)]
pub struct ChooseColorRequest {
    color: String,
}

pub async fn create_game(
    State(state): State<AppState>,
    Json(req): Json<CreateGameRequest>,
) -> impl IntoResponse {
    match UnoGame::new(req.player_names) {
        Ok(game) => match state.session_manager.create_session(game) {
            Ok(session) => {
                let response = GameResponse::from_session(&session);
                (StatusCode::CREATED, Json(response)).into_response()
            }
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

pub async fn list_games(State(state): State<AppState>) -> impl IntoResponse {
    match state.session_manager.list_sessions() {
        Ok(sessions) => Json(sessions).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_game(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.session_manager.load_session(&id) {
        Ok(session) => {
            let response = GameResponse::from_session(&session);
            Json(response).into_response()
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

pub async fn delete_game(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.session_manager.delete_session(&id) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

pub async fn play_card(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<PlayCardRequest>,
) -> impl IntoResponse {
    match state.session_manager.load_session(&id) {
        Ok(mut session) => {
            match session
                .game
                .play_card(session.game.current_turn, req.card_index)
            {
                Ok(event) => {
                    session.game.next_turn();
                    if let Err(e) = session.save(&state.session_manager.sessions_dir) {
                        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                    }
                    Json(event).into_response()
                }
                Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            }
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

pub async fn draw_card(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    match state.session_manager.load_session(&id) {
        Ok(mut session) => match session.game.draw_card(session.game.current_turn) {
            Ok(event) => {
                session.game.next_turn();
                if let Err(e) = session.save(&state.session_manager.sessions_dir) {
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
                Json(event).into_response()
            }
            Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        },
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

pub async fn choose_color(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ChooseColorRequest>,
) -> impl IntoResponse {
    match state.session_manager.load_session(&id) {
        Ok(mut session) => {
            let color = match req.color.to_lowercase().as_str() {
                "red" => Color::Red,
                "green" => Color::Green,
                "blue" => Color::Blue,
                "yellow" => Color::Yellow,
                _ => return (StatusCode::BAD_REQUEST, "Invalid color".to_string()).into_response(),
            };

            if let Some(top_card) = session.game.discard_pile.last_mut() {
                top_card.color = color;
                if let Err(e) = session.save(&state.session_manager.sessions_dir) {
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
                StatusCode::OK.into_response()
            } else {
                (
                    StatusCode::BAD_REQUEST,
                    "No card in discard pile".to_string(),
                )
                    .into_response()
            }
        }
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

impl GameResponse {
    fn from_session(session: &GameSession) -> Self {
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

pub async fn start_api_server(sessions_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let session_manager = SessionManager::new(sessions_dir)?;
    let state = AppState { session_manager };

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/games", post(create_game))
        .route("/games", get(list_games))
        .route("/games/{id}", get(get_game))
        .route("/games/{id}", post(delete_game))
        .route("/games/{id}/play", post(play_card))
        .route("/games/{id}/draw", post(draw_card))
        .route("/games/{id}/color", post(choose_color))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("API server running on http://127.0.0.1:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
