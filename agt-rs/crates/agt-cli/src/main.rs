mod commands;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agt", about = "Agent-native todo/project management", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project in the current directory
    Init {
        /// Project name
        #[arg(long)]
        name: Option<String>,
        /// Project prefix (e.g. "ATL")
        #[arg(long)]
        prefix: Option<String>,
    },
    /// Add a new todo
    Add {
        /// Todo title
        title: String,
        /// Priority (none, low, medium, high, urgent)
        #[arg(long)]
        priority: Option<String>,
        /// Status (none, todo, in_progress, completed, archived, wont_do, needs_elaboration)
        #[arg(long)]
        status: Option<String>,
        /// Difficulty (none, easy, medium, hard)
        #[arg(long)]
        difficulty: Option<String>,
        /// Assignee name or ID
        #[arg(long)]
        assignee: Option<String>,
        /// Description
        #[arg(long)]
        description: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List todos
    List {
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by assignee
        #[arg(long)]
        assignee: Option<String>,
        /// Filter by priority
        #[arg(long)]
        priority: Option<String>,
        /// Search title and description
        #[arg(long)]
        search: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show a single todo
    Show {
        /// Todo reference (e.g. "ATL-1" or "1")
        reference: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Update a todo
    Update {
        /// Todo reference (e.g. "ATL-1" or "1")
        reference: String,
        /// New title
        #[arg(long)]
        title: Option<String>,
        /// New status
        #[arg(long)]
        status: Option<String>,
        /// New priority
        #[arg(long)]
        priority: Option<String>,
        /// New difficulty
        #[arg(long)]
        difficulty: Option<String>,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Delete a todo
    Delete {
        /// Todo reference
        reference: String,
    },
    /// Assign a todo to a member
    Assign {
        /// Todo reference
        reference: String,
        /// Member name or ID
        member: String,
    },
    /// Unassign a todo
    Unassign {
        /// Todo reference
        reference: String,
    },
    /// Add a comment to a todo
    Comment {
        /// Todo reference
        reference: String,
        /// Comment text
        text: String,
    },
    /// Create/manage a git branch for a todo
    Branch {
        /// Todo reference
        reference: String,
    },
    /// Manage project members
    Member {
        #[command(subcommand)]
        action: MemberAction,
    },
    /// Show project config
    Config,
    /// Start the web dashboard server
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "3000")]
        port: u16,
    },
    /// Manage the freeform inbox
    Inbox {
        /// Action: show, append, clear
        action: Option<String>,
        /// Text to append (when action is "append")
        text: Option<String>,
    },
    /// Show audit log
    Log {
        /// Maximum number of entries
        #[arg(long, short = 'n')]
        limit: Option<usize>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Git merge driver for .automerge files (called by git)
    MergeDriver {
        /// Base file path
        base: String,
        /// Ours file path
        ours: String,
        /// Theirs file path
        theirs: String,
    },
}

#[derive(Subcommand)]
enum MemberAction {
    /// Add a new member
    Add {
        /// Member name
        name: String,
        /// Role (owner, member, agent)
        #[arg(long, default_value = "member")]
        role: String,
        /// Email address
        #[arg(long)]
        email: Option<String>,
        /// Agent provider (claude-code, opencode, custom)
        #[arg(long)]
        provider: Option<String>,
        /// Agent model identifier
        #[arg(long)]
        model: Option<String>,
    },
    /// List all members
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove a member
    Remove {
        /// Member name or ID
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name, prefix } => commands::init::run(name, prefix),
        Commands::Add {
            title,
            priority,
            status,
            difficulty,
            assignee,
            description,
            json,
        } => commands::add::run(title, priority, status, difficulty, assignee, description, json),
        Commands::List {
            status,
            assignee,
            priority,
            search,
            json,
        } => commands::list::run(status, assignee, priority, search, json),
        Commands::Show { reference, json } => commands::show::run(reference, json),
        Commands::Update {
            reference,
            title,
            status,
            priority,
            difficulty,
            description,
            json,
        } => commands::update::run(reference, title, status, priority, difficulty, description, json),
        Commands::Delete { reference } => commands::delete::run(reference),
        Commands::Assign { reference, member } => commands::assign::run(reference, member),
        Commands::Unassign { reference } => commands::unassign::run(reference),
        Commands::Comment { reference, text } => commands::comment::run(reference, text),
        Commands::Branch { reference } => commands::branch::run(reference),
        Commands::Member { action } => match action {
            MemberAction::Add {
                name,
                role,
                email,
                provider,
                model,
            } => commands::member::add(name, role, email, provider, model),
            MemberAction::List { json } => commands::member::list(json),
            MemberAction::Remove { name } => commands::member::remove(name),
        },
        Commands::Config => commands::config::run(),
        Commands::Serve { port } => commands::serve::run(port),
        Commands::Inbox { action, text } => commands::inbox::run(action, text),
        Commands::Log { limit, json } => commands::log::run(limit, json),
        Commands::MergeDriver {
            base,
            ours,
            theirs,
        } => commands::merge_driver::run(base, ours, theirs),
    }
}
