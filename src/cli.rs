use clap::{Parser, Subcommand, ValueEnum};

/// Command-line interface for controlling a running spice2x instance.
#[derive(Debug, Parser)]
#[command(name = "spice2x-cli", version, about)]
pub struct Cli {
    /// Host address of the spice2x instance
    #[arg(long, default_value = "localhost", global = true)]
    pub host: String,

    /// Port of the SpiceAPI server
    #[arg(long, default_value_t = 1337, global = true)]
    pub port: u16,

    /// Password for RC4 encryption (empty for plaintext)
    #[arg(long, default_value = "", global = true)]
    pub password: String,

    /// Output format
    #[arg(long, default_value = "json", global = true)]
    pub format: OutputFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Read and write digital button states
    Buttons {
        #[command(subcommand)]
        action: ButtonsAction,
    },
    /// Read and write analog inputs
    Analogs {
        #[command(subcommand)]
        action: AnalogsAction,
    },
    /// Manage coin count
    Coin {
        #[command(subcommand)]
        action: CoinAction,
    },
    /// Screenshot capture
    Capture {
        #[command(subcommand)]
        action: CaptureAction,
    },
    /// Query system information
    Info {
        #[command(subcommand)]
        action: InfoAction,
    },
    /// Insert a virtual card
    Card {
        #[command(subcommand)]
        action: CardAction,
    },
    /// Keypad input control
    Keypads {
        #[command(subcommand)]
        action: KeypadsAction,
    },
    /// Read light states
    Lights {
        #[command(subcommand)]
        action: LightsAction,
    },
    /// Process lifecycle control
    Control {
        #[command(subcommand)]
        action: ControlAction,
    },
}

#[derive(Debug, Subcommand)]
pub enum ButtonsAction {
    /// Read all button states
    Read,
    /// Set a button's state
    Write {
        /// Button name
        name: String,
        /// State value (0.0 or 1.0)
        state: f64,
    },
    /// Reset buttons to default state
    WriteReset {
        /// Button names to reset (all if omitted)
        names: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum AnalogsAction {
    /// Read all analog values
    Read,
    /// Set an analog's value
    Write {
        /// Analog name
        name: String,
        /// Analog value
        value: f64,
    },
    /// Reset analogs to default state
    WriteReset {
        /// Analog names to reset (all if omitted)
        names: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum CoinAction {
    /// Get current coin count
    Get,
    /// Set coin count
    Set {
        /// Coin amount
        amount: u32,
    },
    /// Insert coins
    Insert {
        /// Number of coins to insert
        #[arg(default_value_t = 1)]
        amount: u32,
    },
}

#[derive(Debug, Subcommand)]
pub enum CaptureAction {
    /// List available screen indices
    GetScreens,
    /// Capture a JPEG screenshot
    GetJpg {
        /// Screen index
        #[arg(long, default_value_t = 0)]
        screen: u32,
        /// JPEG quality (1-100)
        #[arg(long, default_value_t = 70)]
        quality: u32,
        /// Image size divisor
        #[arg(long, default_value_t = 1)]
        divide: u32,
        /// Output file path (auto-generated if omitted)
        #[arg(long, conflicts_with = "output_folder")]
        output_path: Option<String>,
        /// Output folder (file is auto-named with timestamp)
        #[arg(long, conflicts_with = "output_path")]
        output_folder: Option<String>,
        /// Output base64 to stdout instead of saving to file
        #[arg(long)]
        base64: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum InfoAction {
    /// AVS system info
    Avs,
    /// Launcher info
    Launcher,
    /// Memory usage info
    Memory,
}

#[derive(Debug, Subcommand)]
pub enum CardAction {
    /// Insert a virtual card
    Insert {
        /// Card reader unit index
        unit: u32,
        /// Card ID string
        card_id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum KeypadsAction {
    /// Get keypad state
    Get {
        /// Keypad index
        keypad: u32,
    },
    /// Write input to keypad
    Write {
        /// Keypad index
        keypad: u32,
        /// Input string
        input: String,
    },
    /// Set individual key values
    Set {
        /// Keypad index
        keypad: u32,
        /// Key characters (0-9, A, D)
        keys: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum LightsAction {
    /// Read light states
    Read {
        /// Light names to read (all if omitted)
        names: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ControlAction {
    /// Raise a signal
    Raise {
        /// Signal name
        signal: String,
    },
    /// Exit the process
    Exit {
        /// Exit code
        #[arg(default_value_t = 0)]
        code: i32,
    },
    /// Restart the process
    Restart,
    /// Shut down the machine
    Shutdown,
    /// Reboot the machine
    Reboot,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Cli {
        Cli::parse_from(args)
    }

    #[test]
    fn default_connection_flags() {
        let cli = parse(&["spice2x-cli", "info", "avs"]);
        assert_eq!(cli.host, "localhost");
        assert_eq!(cli.port, 1337);
        assert_eq!(cli.password, "");
        assert!(matches!(cli.format, OutputFormat::Json));
    }

    #[test]
    fn custom_connection_flags() {
        let cli = parse(&[
            "spice2x-cli", "--host", "192.168.1.10", "--port", "9999",
            "--password", "secret", "--format", "text", "info", "avs",
        ]);
        assert_eq!(cli.host, "192.168.1.10");
        assert_eq!(cli.port, 9999);
        assert_eq!(cli.password, "secret");
        assert!(matches!(cli.format, OutputFormat::Text));
    }

    #[test]
    fn buttons_read() {
        let cli = parse(&["spice2x-cli", "buttons", "read"]);
        assert!(matches!(cli.command, Commands::Buttons { action: ButtonsAction::Read }));
    }

    #[test]
    fn buttons_write() {
        let cli = parse(&["spice2x-cli", "buttons", "write", "BT_A", "1.0"]);
        match cli.command {
            Commands::Buttons { action: ButtonsAction::Write { name, state } } => {
                assert_eq!(name, "BT_A");
                assert_eq!(state, 1.0);
            }
            _ => panic!("expected buttons write"),
        }
    }

    #[test]
    fn buttons_write_reset_no_args() {
        let cli = parse(&["spice2x-cli", "buttons", "write-reset"]);
        match cli.command {
            Commands::Buttons { action: ButtonsAction::WriteReset { names } } => {
                assert!(names.is_empty());
            }
            _ => panic!("expected buttons write-reset"),
        }
    }

    #[test]
    fn buttons_write_reset_with_names() {
        let cli = parse(&["spice2x-cli", "buttons", "write-reset", "BT_A", "BT_B"]);
        match cli.command {
            Commands::Buttons { action: ButtonsAction::WriteReset { names } } => {
                assert_eq!(names, vec!["BT_A", "BT_B"]);
            }
            _ => panic!("expected buttons write-reset"),
        }
    }

    #[test]
    fn coin_insert_default() {
        let cli = parse(&["spice2x-cli", "coin", "insert"]);
        match cli.command {
            Commands::Coin { action: CoinAction::Insert { amount } } => assert_eq!(amount, 1),
            _ => panic!("expected coin insert"),
        }
    }

    #[test]
    fn coin_insert_custom() {
        let cli = parse(&["spice2x-cli", "coin", "insert", "5"]);
        match cli.command {
            Commands::Coin { action: CoinAction::Insert { amount } } => assert_eq!(amount, 5),
            _ => panic!("expected coin insert"),
        }
    }

    #[test]
    fn capture_get_jpg_defaults() {
        let cli = parse(&["spice2x-cli", "capture", "get-jpg"]);
        match cli.command {
            Commands::Capture { action: CaptureAction::GetJpg { screen, quality, divide, output_path, output_folder, base64 } } => {
                assert_eq!(screen, 0);
                assert_eq!(quality, 70);
                assert_eq!(divide, 1);
                assert!(output_path.is_none());
                assert!(output_folder.is_none());
                assert!(!base64);
            }
            _ => panic!("expected capture get-jpg"),
        }
    }

    #[test]
    fn capture_get_jpg_all_flags() {
        let cli = parse(&[
            "spice2x-cli", "capture", "get-jpg",
            "--screen", "1", "--quality", "90", "--divide", "2",
            "--output-path", "shot.jpg", "--base64",
        ]);
        match cli.command {
            Commands::Capture { action: CaptureAction::GetJpg { screen, quality, divide, output_path, output_folder, base64 } } => {
                assert_eq!(screen, 1);
                assert_eq!(quality, 90);
                assert_eq!(divide, 2);
                assert_eq!(output_path.as_deref(), Some("shot.jpg"));
                assert!(output_folder.is_none());
                assert!(base64);
            }
            _ => panic!("expected capture get-jpg"),
        }
    }

    #[test]
    fn capture_get_jpg_output_folder() {
        let cli = parse(&[
            "spice2x-cli", "capture", "get-jpg", "--output-folder", "/tmp/caps",
        ]);
        match cli.command {
            Commands::Capture { action: CaptureAction::GetJpg { output_path, output_folder, .. } } => {
                assert!(output_path.is_none());
                assert_eq!(output_folder.as_deref(), Some("/tmp/caps"));
            }
            _ => panic!("expected capture get-jpg"),
        }
    }

    #[test]
    fn card_insert() {
        let cli = parse(&["spice2x-cli", "card", "insert", "0", "E004123456789ABC"]);
        match cli.command {
            Commands::Card { action: CardAction::Insert { unit, card_id } } => {
                assert_eq!(unit, 0);
                assert_eq!(card_id, "E004123456789ABC");
            }
            _ => panic!("expected card insert"),
        }
    }

    #[test]
    fn control_exit_default() {
        let cli = parse(&["spice2x-cli", "control", "exit"]);
        match cli.command {
            Commands::Control { action: ControlAction::Exit { code } } => assert_eq!(code, 0),
            _ => panic!("expected control exit"),
        }
    }

    #[test]
    fn lights_read_with_filter() {
        let cli = parse(&["spice2x-cli", "lights", "read", "TOP", "SIDE"]);
        match cli.command {
            Commands::Lights { action: LightsAction::Read { names } } => {
                assert_eq!(names, vec!["TOP", "SIDE"]);
            }
            _ => panic!("expected lights read"),
        }
    }

    #[test]
    fn keypads_set() {
        let cli = parse(&["spice2x-cli", "keypads", "set", "0", "1", "2", "A"]);
        match cli.command {
            Commands::Keypads { action: KeypadsAction::Set { keypad, keys } } => {
                assert_eq!(keypad, 0);
                assert_eq!(keys, vec!["1", "2", "A"]);
            }
            _ => panic!("expected keypads set"),
        }
    }
}
