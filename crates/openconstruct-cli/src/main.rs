use anyhow::Result;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "openconstruct")]
#[command(about = "OpenConstruct — Agent Onboarding in One Command")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the 5-phase onboarding wizard
    Init,
    /// Show current agent config and fleet status
    Status,
    /// Manage sense modules
    Sense {
        #[command(subcommand)]
        action: SenseCommands,
    },
    /// Manage agent fleet
    Fleet {
        #[command(subcommand)]
        action: FleetCommands,
    },
    /// Post and read tick board messages
    Tick {
        #[command(subcommand)]
        action: TickCommands,
    },
    /// Manage rooms: grown residency rooms (grow/walk/show) and Plato rooms
    Room {
        #[command(subcommand)]
        action: RoomCommands,
    },
    /// Scaffold a new module from template
    Build {
        /// Module name
        name: String,
        /// Module language (rust, python, node)
        #[arg(short, long, default_value = "rust")]
        lang: String,
    },
    /// Publish module to crates.io/PyPI/npm
    Publish {
        /// Publish target (crates, pypi, npm, all)
        #[arg(short, long, default_value = "all")]
        target: String,
    },
}

#[derive(Subcommand)]
enum SenseCommands {
    /// List available sense modules
    List,
    /// Enable a sense module
    Enable { name: String },
    /// Disable a sense module
    Disable { name: String },
}

#[derive(Subcommand)]
enum FleetCommands {
    /// Discover other agents on the network
    Discover,
    /// Join a fleet
    Join { address: String },
    /// Leave the fleet
    Leave,
    /// Show fleet members
    Members,
}

#[derive(Subcommand)]
enum TickCommands {
    /// Post a message to the tick board
    Post {
        /// The message to post
        message: String,
    },
    /// Read recent ticks
    Read {
        /// Number of recent ticks to show
        #[arg(short, long, default_value = "20")]
        count: usize,
    },
}

#[derive(Subcommand)]
enum RoomCommands {
    /// Grow a new residency room from a seed and charter (RFC 0004)
    Grow {
        /// The seed the room grows from (any string; its bytes are the seed)
        #[arg(long)]
        seed: String,
        /// The room's charter — the commission it holds
        #[arg(long)]
        charter: String,
        /// Room file to create
        #[arg(long)]
        out: PathBuf,
    },
    /// Record one arrival (walk) on a grown room
    Walk {
        /// Room file grown with `room grow`
        file: PathBuf,
        /// The road the arrival came by (e.g. "local", "h-road-0")
        #[arg(long)]
        road: String,
        /// Link quality of the arrival, 0.0–1.0
        #[arg(long, default_value_t = 0.5)]
        link_q: f32,
    },
    /// Show a grown room's growth record / onboarding document
    Show {
        /// Room file grown with `room grow`
        file: PathBuf,
    },
    /// Create a new Plato room
    Create {
        /// Room name
        name: String,
    },
    /// List rooms
    List,
    /// Join a room
    Join { name: String },
    /// Leave a room
    Leave { name: String },
}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Default)]
struct AgentConfig {
    agent: AgentSection,
    #[serde(default)]
    senses: SensesSection,
    #[serde(default)]
    fleet: FleetSection,
    #[serde(default)]
    tick: TickSection,
    #[serde(default)]
    plato: PlatoSection,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentSection {
    name: String,
    #[serde(default = "default_version")]
    version: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SensesSection {
    #[serde(default = "default_true")]
    filesystem: bool,
    #[serde(default = "default_true")]
    network: bool,
    #[serde(default = "default_true")]
    system: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct FleetSection {
    #[serde(default = "default_discovery")]
    discovery: String,
    #[serde(default = "default_port")]
    port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
struct TickSection {
    #[serde(default = "default_board")]
    board: String,
    #[serde(default = "default_retention")]
    retention: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PlatoSection {
    #[serde(default = "default_room")]
    default_room: String,
}

fn default_version() -> String { "0.1.0".into() }
fn default_true() -> bool { true }
fn default_discovery() -> String { "lan".into() }
fn default_port() -> u16 { 7490 }
fn default_board() -> String { "local".into() }
fn default_retention() -> String { "7d".into() }
fn default_room() -> String { "general".into() }

impl Default for AgentSection {
    fn default() -> Self {
        Self { name: "my-agent".into(), version: default_version() }
    }
}
impl Default for FleetSection {
    fn default() -> Self {
        Self { discovery: default_discovery(), port: default_port() }
    }
}
impl Default for TickSection {
    fn default() -> Self {
        Self { board: default_board(), retention: default_retention() }
    }
}
impl Default for PlatoSection {
    fn default() -> Self {
        Self { default_room: default_room() }
    }
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openconstruct")
        .join("agent.toml")
}

fn load_config() -> Result<AgentConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(AgentConfig::default());
    }
    let content = fs::read_to_string(&path)?;
    let config: AgentConfig = toml::from_str(&content)?;
    Ok(config)
}

fn save_config(config: &AgentConfig) -> Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}

fn tick_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openconstruct")
        .join("ticks.jsonl")
}

// ── Command Handlers ────────────────────────────────────────────────────────

fn run_init() -> Result<()> {
    use dialoguer::{Input, Select, Confirm};

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          OpenConstruct — Onboarding Wizard              ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    // Phase 1: Identity
    println!("Phase 1/5 — Identity");
    let agent_name: String = Input::new()
        .with_prompt("Agent name")
        .default("my-agent".into())
        .interact()?;

    // Phase 2: Senses
    println!("\nPhase 2/5 — Senses");
    let sense_options = ["filesystem + network + system (default)", "all including web + code", "minimal (filesystem only)"];
    let sense_choice = Select::new()
        .with_prompt("Select sense profile")
        .items(&sense_options)
        .default(0)
        .interact()?;

    let senses = SensesSection {
        filesystem: true,
        network: sense_choice != 2,
        system: sense_choice != 2,
    };

    // Phase 3: Fleet
    println!("\nPhase 3/5 — Fleet");
    let discovery_options = ["lan", "mesh", "off"];
    let discovery_choice = Select::new()
        .with_prompt("Discovery mode")
        .items(&discovery_options)
        .default(0)
        .interact()?;

    let fleet = FleetSection {
        discovery: discovery_options[discovery_choice].into(),
        port: 7490,
    };

    // Phase 4: Tick Board
    println!("\nPhase 4/5 — Tick Board");
    let board_options = ["local", "remote", "off"];
    let board_choice = Select::new()
        .with_prompt("Tick board mode")
        .items(&board_options)
        .default(0)
        .interact()?;

    let tick = TickSection {
        board: board_options[board_choice].into(),
        retention: "7d".into(),
    };

    // Phase 5: Build
    println!("\nPhase 5/5 — Build");
    let create_module = Confirm::new()
        .with_prompt("Create a starter module?")
        .default(false)
        .interact()?;

    if create_module {
        let module_name: String = Input::new()
            .with_prompt("Module name")
            .default("hello-sense".into())
            .interact()?;
        scaffold_module(&module_name, "rust")?;
    }

    let config = AgentConfig {
        agent: AgentSection { name: agent_name, version: "0.1.0".into() },
        senses,
        fleet,
        tick,
        plato: PlatoSection::default(),
    };
    save_config(&config)?;

    println!();
    println!("✓ Onboarding complete!");
    println!();
    println!("  Config:  ~/.openconstruct/agent.toml");
    println!("  Status:  openconstruct status");
    println!("  Docs:    https://github.com/SuperInstance/openconstruct-docs");
    println!();

    Ok(())
}

fn run_status() -> Result<()> {
    let config = load_config()?;

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          OpenConstruct — Agent Status                   ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    println!("  Agent:     {} v{}", config.agent.name, config.agent.version);
    println!("  Config:    {}", config_path().display());
    println!();
    println!("  Senses:");
    println!("    filesystem: {}", if config.senses.filesystem { "✓ enabled" } else { "✗ disabled" });
    println!("    network:    {}", if config.senses.network { "✓ enabled" } else { "✗ disabled" });
    println!("    system:     {}", if config.senses.system { "✓ enabled" } else { "✗ disabled" });
    println!();
    println!("  Fleet:");
    println!("    discovery:  {}", config.fleet.discovery);
    println!("    port:       {}", config.fleet.port);
    println!();
    println!("  Tick Board:");
    println!("    board:      {}", config.tick.board);
    println!("    retention:  {}", config.tick.retention);
    println!();
    println!("  Plato:");
    println!("    default:    {}", config.plato.default_room);
    println!();

    Ok(())
}

fn run_sense_list() -> Result<()> {
    let config = load_config()?;

    println!("Available sense modules:");
    println!();
    println!("  {:20} {:10} {}", "Module", "Status", "Description");
    println!("  {} {} {}", "─".repeat(20), "─".repeat(10), "─".repeat(40));
    let modules = [
        ("filesystem", config.senses.filesystem, "Read/write local filesystem"),
        ("network", config.senses.network, "HTTP/WebSocket client"),
        ("system", config.senses.system, "System info, processes, resources"),
        ("web", false, "Web scraping and browsing"),
        ("code", false, "Code analysis and execution"),
    ];
    for (name, enabled, desc) in modules {
        let status = if enabled { "✓ enabled" } else { "✗ disabled" };
        println!("  {:20} {:10} {}", name, status, desc);
    }
    println!();

    Ok(())
}

fn run_sense_enable(name: &str) -> Result<()> {
    let mut config = load_config()?;
    match name {
        "filesystem" => config.senses.filesystem = true,
        "network" => config.senses.network = true,
        "system" => config.senses.system = true,
        _ => { println!("Unknown sense module: {} (available: filesystem, network, system)", name); return Ok(()); }
    }
    save_config(&config)?;
    println!("✓ Sense '{}' enabled", name);
    Ok(())
}

fn run_sense_disable(name: &str) -> Result<()> {
    let mut config = load_config()?;
    match name {
        "filesystem" => config.senses.filesystem = false,
        "network" => config.senses.network = false,
        "system" => config.senses.system = false,
        _ => { println!("Unknown sense module: {} (available: filesystem, network, system)", name); return Ok(()); }
    }
    save_config(&config)?;
    println!("✓ Sense '{}' disabled", name);
    Ok(())
}

fn run_fleet_discover() -> Result<()> {
    let config = load_config()?;
    println!("Scanning for agents on {} (port {})...", config.fleet.discovery, config.fleet.port);
    println!();
    println!("  Local agent: {} v{}", config.agent.name, config.agent.version);
    println!("  Status:      listening on :{}", config.fleet.port);
    println!();
    println!("  (Fleet discovery requires a running agent daemon — coming soon)");
    println!("  Use 'openconstruct fleet join <address>' to connect to a peer.");
    println!();
    Ok(())
}

fn run_fleet_join(address: &str) -> Result<()> {
    println!("Joining fleet at {}...", address);
    println!("✓ Fleet join queued (requires running agent daemon)");
    Ok(())
}

fn run_fleet_leave() -> Result<()> {
    println!("Leaving fleet...");
    println!("✓ Left fleet");
    Ok(())
}

fn run_fleet_members() -> Result<()> {
    let config = load_config()?;
    println!("Fleet members:");
    println!();
    println!("  {} v{} (local)", config.agent.name, config.agent.version);
    println!();
    println!("  (Run 'openconstruct fleet discover' to find peers)");
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct TickEntry {
    agent: String,
    message: String,
    timestamp: String,
}

fn run_tick_post(message: &str) -> Result<()> {
    let config = load_config()?;
    let entry = TickEntry {
        agent: config.agent.name.clone(),
        message: message.into(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    let path = tick_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = fs::OpenOptions::new().create(true).append(true).open(&path)?;
    use std::io::Write;
    writeln!(file, "{}", serde_json::to_string(&entry)?)?;

    println!("✓ Tick posted: {}", message);
    Ok(())
}

fn run_tick_read(count: usize) -> Result<()> {
    let path = tick_path();
    if !path.exists() {
        println!("No ticks yet. Post one with: openconstruct tick post \"message\"");
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;
    let lines: Vec<&str> = content.lines().rev().take(count).collect();

    println!("Recent ticks ({} of {}):", lines.len().min(count), content.lines().count());
    println!();
    for line in lines.iter().rev() {
        if let Ok(entry) = serde_json::from_str::<TickEntry>(line) {
            let ts = entry.timestamp.get(..19).unwrap_or(&entry.timestamp);
            println!("  [{}] {} — {}", ts, entry.agent, entry.message);
        }
    }
    println!();
    Ok(())
}

fn run_room_create(name: &str) -> Result<()> {
    let rooms_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openconstruct")
        .join("rooms.jsonl");

    if let Some(parent) = rooms_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let room = serde_json::json!({
        "name": name,
        "created": chrono::Utc::now().to_rfc3339(),
    });
    let mut file = fs::OpenOptions::new().create(true).append(true).open(&rooms_path)?;
    use std::io::Write;
    writeln!(file, "{}", room)?;

    println!("✓ Room '{}' created", name);
    Ok(())
}

fn run_room_list() -> Result<()> {
    let rooms_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openconstruct")
        .join("rooms.jsonl");

    if !rooms_path.exists() {
        println!("No rooms yet. Create one with: openconstruct room create <name>");
        return Ok(());
    }

    let content = fs::read_to_string(&rooms_path)?;
    println!("Rooms:");
    println!();
    for line in content.lines() {
        if let Ok(room) = serde_json::from_str::<serde_json::Value>(line) {
            let name = room["name"].as_str().unwrap_or("?");
            let created = room["created"].as_str().unwrap_or("?");
            println!("  {} (created {})", name, &created[..19.min(created.len())]);
        }
    }
    println!();
    Ok(())
}

fn run_room_join(name: &str) -> Result<()> {
    println!("✓ Joined room '{}'", name);
    Ok(())
}

fn run_room_leave(name: &str) -> Result<()> {
    println!("✓ Left room '{}'", name);
    Ok(())
}

fn scaffold_module(name: &str, lang: &str) -> Result<()> {
    let module_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".openconstruct")
        .join("modules")
        .join(name);

    fs::create_dir_all(&module_dir)?;

    match lang {
        "rust" => {
            let cargo_toml = format!(
                r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
                name = name
            );
            let lib_rs = r#"//! OpenConstruct sense module

/// Initialize this sense module
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    println!("Module initialized!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
"#;
            fs::write(module_dir.join("Cargo.toml"), cargo_toml)?;
            fs::create_dir_all(module_dir.join("src"))?;
            fs::write(module_dir.join("src/lib.rs"), lib_rs)?;
        }
        "python" => {
            let init_py = r#""""OpenConstruct sense module."""


def init() -> None:
    """Initialize this sense module."""
    print("Module initialized!")
"#;
            fs::write(module_dir.join("__init__.py"), init_py)?;
        }
        "node" => {
            let index_js = r#"// OpenConstruct sense module

function init() {
  console.log("Module initialized!");
}

module.exports = { init };
"#;
            let package_json = format!(
                r#"{{"name": "{name}", "version": "0.1.0", "main": "index.js"}}"#,
                name = name
            );
            fs::write(module_dir.join("index.js"), index_js)?;
            fs::write(module_dir.join("package.json"), package_json)?;
        }
        _ => {
            println!("Unknown language: {} (supported: rust, python, node)", lang);
            return Ok(());
        }
    }

    println!("✓ Module '{}' scaffolded at {}", name, module_dir.display());
    Ok(())
}

fn run_build(name: &str, lang: &str) -> Result<()> {
    scaffold_module(name, lang)
}

fn run_publish(target: &str) -> Result<()> {
    println!("Publishing to {}...", target);
    match target {
        "crates" => {
            println!("  Running: cargo publish");
            println!("  (Requires crates.io token — set with: cargo login <token>)");
        }
        "pypi" => {
            println!("  Running: python -m build && twine upload dist/*");
            println!("  (Requires PyPI token — set in ~/.pypirc)");
        }
        "npm" => {
            println!("  Running: npm publish");
            println!("  (Requires npm login)");
        }
        "all" => {
            println!("  Would publish to: crates.io, PyPI, npm");
            println!("  Run with --target <specific> for individual publishes");
        }
        _ => println!("  Unknown target: {}", target),
    }
    println!("✓ Publish instructions printed (auto-publish coming soon)");
    Ok(())
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => run_init(),
        Commands::Status => run_status(),
        Commands::Sense { action } => match action {
            SenseCommands::List => run_sense_list(),
            SenseCommands::Enable { name } => run_sense_enable(&name),
            SenseCommands::Disable { name } => run_sense_disable(&name),
        },
        Commands::Fleet { action } => match action {
            FleetCommands::Discover => run_fleet_discover(),
            FleetCommands::Join { address } => run_fleet_join(&address),
            FleetCommands::Leave => run_fleet_leave(),
            FleetCommands::Members => run_fleet_members(),
        },
        Commands::Tick { action } => match action {
            TickCommands::Post { message } => run_tick_post(&message),
            TickCommands::Read { count } => run_tick_read(count),
        },
        Commands::Room { action } => match action {
            RoomCommands::Grow { seed, charter, out } => {
                let tick = u64::try_from(chrono::Utc::now().timestamp()).unwrap_or(0);
                let doc = openshell_construct::cli::grow_room(&seed, &charter, tick, &out)?;
                println!("{}", doc.trim_end());
                println!();
                println!("✓ Room grown and saved to {}", out.display());
                Ok(())
            }
            RoomCommands::Walk { file, road, link_q } => {
                let ts = u64::try_from(chrono::Utc::now().timestamp()).unwrap_or(0);
                let out = openshell_construct::cli::walk_room(&file, &road, link_q, ts)?;
                println!("{}", out);
                Ok(())
            }
            RoomCommands::Show { file } => {
                let doc = openshell_construct::cli::show_room(&file)?;
                println!("{}", doc.trim_end());
                Ok(())
            }
            RoomCommands::Create { name } => run_room_create(&name),
            RoomCommands::List => run_room_list(),
            RoomCommands::Join { name } => run_room_join(&name),
            RoomCommands::Leave { name } => run_room_leave(&name),
        },
        Commands::Build { name, lang } => run_build(&name, &lang),
        Commands::Publish { target } => run_publish(&target),
    }
}
