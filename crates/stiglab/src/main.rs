mod runner;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

use stiglab_agent::config::AgentConfig;
use stiglab_server::config::ServerConfig;
use stiglab_server::{db, state::AppState};

#[derive(Parser)]
#[command(
    name = "stiglab",
    about = "Stiglab – distributed AI agent orchestration"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run as server: serves UI, API, and optionally executes tasks locally
    Server {
        /// Disable the built-in task runner (server becomes API-only, requires external agents)
        #[arg(long, env = "STIGLAB_NO_RUNNER")]
        no_runner: bool,

        /// Maximum concurrent sessions for the built-in runner
        #[arg(long, env = "STIGLAB_MAX_SESSIONS", default_value = "4")]
        max_sessions: u32,

        /// Command to execute for agent sessions
        #[arg(long, env = "STIGLAB_AGENT_COMMAND", default_value = "claude")]
        agent_command: String,

        /// Name for the built-in runner node
        #[arg(long, env = "STIGLAB_NODE_NAME")]
        node_name: Option<String>,
    },

    /// Run as agent: connects to a server and executes tasks
    Agent(AgentArgs),
}

#[derive(Parser)]
struct AgentArgs {
    /// WebSocket URL of the server
    #[arg(
        long,
        short,
        env = "STIGLAB_SERVER_URL",
        default_value = "ws://localhost:3000/agent/ws"
    )]
    server: String,

    /// Name of this node
    #[arg(long, short, env = "STIGLAB_NODE_NAME")]
    name: Option<String>,

    /// Maximum concurrent sessions
    #[arg(long, short, env = "STIGLAB_MAX_SESSIONS", default_value = "4")]
    max_sessions: u32,

    /// Command to execute for agent sessions
    #[arg(long, env = "STIGLAB_AGENT_COMMAND", default_value = "claude")]
    agent_command: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Server {
            no_runner,
            max_sessions,
            agent_command,
            node_name,
        } => run_server(no_runner, max_sessions, agent_command, node_name).await,
        Command::Agent(args) => run_agent(args).await,
    }
}

async fn run_server(
    no_runner: bool,
    max_sessions: u32,
    agent_command: String,
    node_name: Option<String>,
) -> anyhow::Result<()> {
    let config = ServerConfig::from_env();
    tracing::info!("starting stiglab server on {}:{}", config.host, config.port);

    tracing::info!("connecting to database...");
    let pool = db::init_pool(&config.database_url).await?;
    tracing::info!("database connected");
    let state = AppState::new(pool.clone());

    // Start built-in runner if enabled
    if !no_runner {
        let runner_node_name = node_name.unwrap_or_else(|| "built-in-runner".to_string());

        tracing::info!(
            "built-in runner enabled: node={runner_node_name}, max_sessions={max_sessions}, command={agent_command}"
        );

        runner::start_built_in_runner(
            &state,
            &pool,
            &runner_node_name,
            max_sessions,
            &agent_command,
        )
        .await?;
    } else {
        tracing::info!("built-in runner disabled, external agents required");
    }

    let app = stiglab_server::build_router(state, &config);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn run_agent(args: AgentArgs) -> anyhow::Result<()> {
    let config = AgentConfig {
        server: args.server,
        name: args.name,
        max_sessions: args.max_sessions,
        agent_command: args.agent_command,
    };

    tracing::info!("stiglab agent starting");
    tracing::info!("  node name: {}", config.node_name());
    tracing::info!("  server: {}", config.server);
    tracing::info!("  max sessions: {}", config.max_sessions);

    loop {
        match stiglab_agent::connection::connect_and_run(config.clone()).await {
            Ok(()) => {
                tracing::info!("connection closed, reconnecting in 5s...");
            }
            Err(e) => {
                tracing::error!("connection error: {e}, reconnecting in 5s...");
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
