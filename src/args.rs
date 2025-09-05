use clap::Parser;

#[derive(clap::Subcommand, Debug, Clone, Copy)]
pub enum Mode {
    Minimax { depth: u32 },
    Mcts { exploration_weight: f32 },
}

/// Command-line arguments for your agent.
///
/// Add any custom parameters you want here.
/// For example: maximum depth, exploration factor, etc.
///
/// Do NOT forget to update your config file accordingly!
///
/// If you want to access runtime information (e.g. agent's color, how much time
/// is given to your agent), check out `agent_bin.rs`.
#[derive(Parser, Debug)]
pub struct EngineArgs {
    #[clap(subcommand)]
    pub mode: Mode,
    // Tutorial on clap derive: https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html
}
