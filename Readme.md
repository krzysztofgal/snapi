Network (Un)playable snake game.

run with: `cargo run` (`--release`)

Game server is available at: `0.0.0.0:3000`

Commands (HTTP)

`GET /snake` - display game state (level)  
`POST /snake/:direction` - change snake movement direction where `:direction` is one of `left`, `right`, `bottom`, `down`, `top`, `up`.

Preview in terminal must have enough space to refresh properly, or it will behave like print to new line on each level render.
