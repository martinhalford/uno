# UNO Game API

Copyright © Martin Halford 2025

A RESTful API implementation of the Uno card game, written in Rust using Axum.

## Features

- Create and manage multiple game sessions
- Play Uno with multiple players
- Support for all standard Uno cards including:
  - Numbered cards (0-9)
  - Action cards (Skip, Reverse, Draw Two)
  - Wild cards
  - Wild Draw Four cards
- Real-time game state updates
- Persistent game sessions

## Setup

1. Ensure you have Rust installed (version 1.70 or later)
2. Clone the repository
3. Run the server:
   ```bash
   cargo run -- --server
   ```

The server will start on `http://127.0.0.1:3000`

## API Endpoints

### Create a New Game

```http
POST /games
Content-Type: application/json

{
    "player_names": ["Alice", "Bob", "Charlie"]
}
```

Response:

```json
{
  "id": "6bc0a81b-5aad-46ae-b3a0-fd7b865d5912",
  "current_turn": 0,
  "players": [
    {
      "id": 0,
      "name": "Alice",
      "hand_size": 7
    },
    {
      "id": 1,
      "name": "Bob",
      "hand_size": 7
    }
  ],
  "discard_pile_top": {
    "color": "Red",
    "card_type": "Number(5)"
  },
  "deck_cards_remaining": 93,
  "pending_draws": 0
}
```

### List All Games

```http
GET /games
```

Response:

```json
["6bc0a81b-5aad-46ae-b3a0-fd7b865d5912", "7cd1b92c-6bde-47df-a2e1-fe8c976d6023"]
```

### Get Game State

```http
GET /games/{id}/state
```

Response:

```json
{
  "id": "6bc0a81b-5aad-46ae-b3a0-fd7b865d5912",
  "current_turn": 0,
  "direction": "Clockwise",
  "players": [
    {
      "id": 0,
      "name": "Alice",
      "hand": [
        [0, { "color": "Blue", "card_type": "Number(5)" }],
        [1, { "color": "Blue", "card_type": "Reverse" }],
        [2, { "color": "Wild", "card_type": "WildDrawFour" }]
      ]
    }
  ],
  "discard_pile_top": {
    "color": "Green",
    "card_type": "Number(8)"
  },
  "deck_cards_remaining": 59,
  "pending_draws": 0
}
```

### Play a Card

```http
POST /games/{id}/play
Content-Type: application/json

{
    "card_index": 2,
    "color": "red"  // Required only for Wild and Wild Draw Four cards
}
```

Response:

```json
{
  "WildDrawFour": {
    "player_id": 0,
    "next_player_id": 1,
    "cards": [],
    "color": "Red"
  }
}
```

### Draw a Card

```http
POST /games/{id}/draw
```

Response:

```json
{
  "CardDrawn": {
    "player_id": 0,
    "card": {
      "color": "Blue",
      "card_type": "Number(3)"
    }
  }
}
```

### Choose Color (for Wild cards)

```http
POST /games/{id}/color
Content-Type: application/json

{
    "color": "red"  // One of: "red", "green", "blue", "yellow"
}
```

### Get Deck Contents

```http
GET /games/{id}/deck
```

Response:

```json
{
  "cards": [
    {
      "color": "Red",
      "card_type": "Number(1)"
    },
    {
      "color": "Blue",
      "card_type": "Skip"
    }
  ]
}
```

### Delete a Game

```http
DELETE /games/{id}
```

## Game Rules

1. Players must play a card that matches either:

   - The color of the top card
   - The number of the top card
   - A Wild or Wild Draw Four card (which can be played on any card)

2. When playing a Wild or Wild Draw Four card, the player must specify the next color.

3. Action cards (Skip, Reverse, Draw Two) can only be played on matching colors.

4. When a Draw Two or Wild Draw Four is played, the next player must draw the specified number of cards.

5. The game continues until one player has no cards left.

## Error Handling

The API returns appropriate HTTP status codes:

- 200: Success
- 201: Game created
- 204: Game deleted
- 400: Bad request (invalid move, missing color for Wild card)
- 404: Game not found

## Development

To run tests:

```bash
cargo test
```

To run in CLI mode (instead of API server):

```bash
cargo run
```
