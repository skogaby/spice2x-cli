mod analogs;
mod buttons;
mod capture;
mod card;
mod coin;
mod control;
mod info;
mod keypads;
mod lights;

use anyhow::Result;
use serde_json::Value;

use crate::cli::Commands;
use crate::protocol::Connection;

/// Dispatch a parsed command to the appropriate handler.
///
/// Returns the response data as a JSON value for formatting.
pub fn execute(conn: &mut Connection, command: Commands) -> Result<Value> {
    match command {
        Commands::Info { action } => info::execute(conn, action),
        Commands::Control { action } => control::execute(conn, action),
        Commands::Buttons { action } => buttons::execute(conn, action),
        Commands::Analogs { action } => analogs::execute(conn, action),
        Commands::Coin { action } => coin::execute(conn, action),
        Commands::Card { action } => card::execute(conn, action),
        Commands::Keypads { action } => keypads::execute(conn, action),
        Commands::Capture { action } => capture::execute(conn, action),
        Commands::Lights { action } => lights::execute(conn, action),
    }
}
