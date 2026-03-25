mod commands;
mod output;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "agt",
    about = "Agent-native todo/project management",
    version,
    subcommand_help_heading = "Commands",
    override_usage = "agt <COMMAND>  [--all for full help]"
)]
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
        /// Status (none, todo, queued, in_progress, completed, archived, wont_do, needs_elaboration)
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
        /// Labels (comma-separated: bug, new_feature, feature_plus)
        #[arg(long)]
        labels: Option<String>,
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
        /// Filter by difficulty (easy, medium, hard)
        #[arg(long)]
        difficulty: Option<String>,
        /// Search title and description
        #[arg(long)]
        search: Option<String>,
        /// Include all todos (archived + won't do)
        #[arg(long)]
        all: bool,
        /// Show only archived todos
        #[arg(long)]
        archived: bool,
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
    /// Edit a todo
    #[command(alias = "update")]
    Edit {
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
        /// Labels (comma-separated: bug, new_feature, feature_plus)
        #[arg(long)]
        labels: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Delete a todo
    Delete {
        /// Todo reference
        reference: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Assign a todo to a member
    Assign {
        /// Todo reference
        reference: String,
        /// Member name or ID
        member: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Unassign a todo
    Unassign {
        /// Todo reference
        reference: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a comment to a todo
    Comment {
        /// Todo reference
        reference: String,
        /// Comment text
        text: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Create a git worktree + branch for a todo
    Branch {
        /// Todo reference
        reference: String,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove a git worktree + branch for a todo
    Unbranch {
        /// Todo reference
        reference: String,
        /// Keep the git branch (only remove worktree)
        #[arg(long)]
        keep_branch: bool,
    },
    /// Run a coding agent against a single todo
    Run {
        /// Todo reference (e.g. "ATL-1" or "1")
        reference: String,
        /// Max budget in USD for the agent
        #[arg(long)]
        budget: Option<f64>,
        /// Print rendered prompt and exit without running
        #[arg(long)]
        dry_run: bool,
    },
    /// Poll for queued todos and dispatch agents (cron-compatible)
    Poll {
        /// Print what would be dispatched without doing it
        #[arg(long)]
        dry_run: bool,
    },
    /// Set todo status to queued (ready for agent dispatch)
    Queue {
        /// Todo references (e.g. "ATL-1" "ATL-2")
        references: Vec<String>,
    },
    /// List active agent runs
    Runs {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Manage project members
    Member {
        #[command(subcommand)]
        action: MemberAction,
    },
    /// Show or update project config
    Config {
        /// Set project name
        #[arg(long)]
        name: Option<String>,
        /// Set issue prefix (e.g. ABC)
        #[arg(long)]
        prefix: Option<String>,
    },
    /// Start the web dashboard server
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "3000")]
        port: u16,
        /// Open the dashboard in the default browser
        #[arg(long)]
        open: bool,
    },
    /// Manage the freeform inbox
    Inbox {
        /// Action: show, append, clear
        action: Option<String>,
        /// Text to append (when action is "append")
        text: Option<String>,
    },
    /// Commit .todo/ changes and optionally push
    Commit {
        /// Push to remote after committing
        #[arg(long)]
        push: bool,
        /// Custom commit message
        #[arg(long, short = 'm')]
        message: Option<String>,
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
        /// Output as JSON
        #[arg(long)]
        json: bool,
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
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Update a member
    Update {
        /// Member name or ID
        name: String,
        /// New name (rename)
        #[arg(long = "name", value_name = "NEW_NAME")]
        new_name: Option<String>,
        /// New role (owner, member, agent)
        #[arg(long)]
        role: Option<String>,
        /// New email address
        #[arg(long)]
        email: Option<String>,
        /// Agent provider (claude-code, opencode, custom)
        #[arg(long)]
        provider: Option<String>,
        /// Agent model identifier
        #[arg(long)]
        model: Option<String>,
    },
}

fn print_full_help() {
    let mut cmd = Cli::command()
        .arg(clap::Arg::new("all").long("all").help("Print help for all subcommands").action(clap::ArgAction::SetTrue));
    cmd.print_help().ok();
    println!("\n");
    for sub in cmd.get_subcommands_mut() {
        if sub.get_name() == "help" {
            continue;
        }
        println!("{}", "─".repeat(60));
        sub.print_help().ok();
        println!("\n");
    }
}

fn main() -> Result<()> {
    // `agt --all` or `agt help --all` prints help for every subcommand.
    // Only trigger when --all appears before any subcommand (i.e. args[1])
    // or after "help", to avoid conflicting with `agt list --all`.
    let args: Vec<String> = std::env::args().collect();
    let has_all = args.iter().any(|a| a == "--all" || a == "-a");
    let first_non_flag = args.iter().skip(1).find(|a| !a.starts_with('-'));
    let is_help_context = first_non_flag.is_none() || first_non_flag.map(|s| s.as_str()) == Some("help");
    if has_all && is_help_context {
        print_full_help();
        std::process::exit(0);
    }

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
            labels,
            json,
        } => commands::add::run(
            title,
            priority,
            status,
            difficulty,
            assignee,
            description,
            labels,
            json,
        ),
        Commands::List {
            status,
            assignee,
            priority,
            difficulty,
            search,
            all,
            archived,
            json,
        } => commands::list::run(status, assignee, priority, difficulty, search, all, archived, json),
        Commands::Show { reference, json } => commands::show::run(reference, json),
        Commands::Edit {
            reference,
            title,
            status,
            priority,
            difficulty,
            description,
            labels,
            json,
        } => commands::update::run(
            reference,
            title,
            status,
            priority,
            difficulty,
            description,
            labels,
            json,
        ),
        Commands::Delete { reference, json } => commands::delete::run(reference, json),
        Commands::Assign {
            reference,
            member,
            json,
        } => commands::assign::run(reference, member, json),
        Commands::Unassign { reference, json } => commands::unassign::run(reference, json),
        Commands::Comment {
            reference,
            text,
            json,
        } => commands::comment::run(reference, text, json),
        Commands::Branch { reference, json } => commands::branch::run(reference, json),
        Commands::Unbranch {
            reference,
            keep_branch,
        } => commands::unbranch::run(reference, keep_branch),
        Commands::Run {
            reference,
            budget,
            dry_run,
        } => commands::run::run(reference, budget, dry_run),
        Commands::Poll { dry_run } => commands::poll::run(dry_run),
        Commands::Queue { references } => commands::queue::run(references),
        Commands::Runs { json } => commands::runs::run(json),
        Commands::Member { action } => match action {
            MemberAction::Add {
                name,
                role,
                email,
                provider,
                model,
                json,
            } => commands::member::add(name, role, email, provider, model, json),
            MemberAction::List { json } => commands::member::list(json),
            MemberAction::Remove { name, json } => commands::member::remove(name, json),
            MemberAction::Update {
                name,
                new_name,
                role,
                email,
                provider,
                model,
            } => commands::member::update(name, new_name, role, email, provider, model),
        },
        Commands::Commit { push, message } => commands::commit::run(push, message),
        Commands::Config { name, prefix } => commands::config::run(name, prefix),
        Commands::Serve { port, open } => commands::serve::run(port, open),
        Commands::Inbox { action, text } => commands::inbox::run(action, text),
        Commands::Log { limit, json } => commands::log::run(limit, json),
        Commands::MergeDriver { base, ours, theirs } => {
            commands::merge_driver::run(base, ours, theirs)
        }
    }
}
