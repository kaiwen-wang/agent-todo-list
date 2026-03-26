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
    #[command(alias = "del")]
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
    /// Manage plan/research files for todos
    Plan {
        #[command(subcommand)]
        action: PlanAction,
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
    #[command(hide = true)]
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
enum PlanAction {
    /// Show a todo's plan
    Show {
        /// Todo reference (e.g. "AGT-58" or "58")
        reference: String,
        /// Prompt to answer questions after viewing
        #[arg(short = 'a', long = "answer")]
        answer: bool,
    },
    /// Create a plan file for a todo
    Init {
        /// Todo reference (e.g. "AGT-58" or "58")
        reference: String,
    },
    /// Append an answer to a todo's plan questions
    Answer {
        /// Todo reference (e.g. "AGT-58" or "58")
        reference: String,
        /// Answer text
        text: String,
    },
    /// Print the plan file path (for scripting)
    Path {
        /// Todo reference (e.g. "AGT-58" or "58")
        reference: String,
    },
    /// Spawn an agent to research and flesh out the plan
    Research {
        /// Todo reference (e.g. "AGT-58" or "58")
        reference: String,
        /// Print the prompt without running the agent
        #[arg(long)]
        dry_run: bool,
    },
    /// Delete a todo's plan file (moves to Trash)
    Trash {
        /// Todo reference (e.g. "AGT-58" or "58")
        reference: String,
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

fn print_grouped_help() {
    let cmd = Cli::command();
    let version = cmd.get_version().unwrap_or("unknown");

    println!("agt {version} — Agent-native todo/project management\n");
    println!("Usage: agt <COMMAND>  [--all for full help]\n");

    let groups: &[(&str, &[&str])] = &[
        ("Todo Management", &["add", "list", "show", "edit", "delete"]),
        (
            "Workflow",
            &["assign", "unassign", "comment", "branch", "unbranch", "plan"],
        ),
        ("Agent Dispatch", &["run", "poll", "queue", "runs"]),
        (
            "Project",
            &["init", "member", "config", "serve", "inbox", "commit", "log"],
        ),
    ];

    for (heading, names) in groups {
        println!("\x1b[1;4m{heading}:\x1b[0m");
        for name in *names {
            if let Some(sub) = cmd.find_subcommand(name) {
                let about = sub.get_about().map(|s| s.to_string()).unwrap_or_default();
                println!("  \x1b[1m{name:<14}\x1b[0m {about}");
            }
        }
        println!();
    }

    println!("\x1b[1;4mOptions:\x1b[0m");
    println!("  \x1b[1m-a, --all\x1b[0m      Print help for all subcommands");
    println!("  \x1b[1m-h, --help\x1b[0m     Print help");
    println!("  \x1b[1m-V, --version\x1b[0m  Print version");
}

fn print_full_help() {
    let mut cmd = Cli::command()
        .arg(
            clap::Arg::new("all")
                .long("all")
                .help("Print help for all subcommands")
                .action(clap::ArgAction::SetTrue),
        );

    // Print the grouped overview first instead of clap's flat list
    print_grouped_help();
    println!("\n{}\n", "━".repeat(60));
    println!("Detailed help for all commands:\n");

    for sub in cmd.get_subcommands_mut() {
        if sub.get_name() == "help" || sub.get_name() == "merge-driver" {
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
    let has_help = args.iter().any(|a| a == "--help" || a == "-h");
    let non_flags: Vec<&String> = args.iter().skip(1).filter(|a| !a.starts_with('-')).collect();
    let first_non_flag = non_flags.first().map(|s| s.as_str());
    let is_help_context =
        first_non_flag.is_none() || first_non_flag == Some("help");
    // `agt help <command>` should fall through to clap's per-command help
    let is_help_for_specific =
        first_non_flag == Some("help") && non_flags.len() > 1;

    if is_help_context && !is_help_for_specific {
        if has_all {
            print_full_help();
        } else if has_help || first_non_flag.is_none() || first_non_flag == Some("help") {
            print_grouped_help();
        }
        // If we matched any of the above, exit. Otherwise fall through to clap.
        if has_all || has_help || first_non_flag.is_none() || first_non_flag == Some("help") {
            std::process::exit(0);
        }
    }

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            use std::io::Write;
            let msg = e.render().to_string();
            let msg = msg.replace(
                "For more information, try '--help'.",
                "For more information, try '--help' or '-h'.",
            );
            if e.use_stderr() {
                let _ = write!(std::io::stderr(), "{msg}");
            } else {
                let _ = write!(std::io::stdout(), "{msg}");
            }
            std::process::exit(e.exit_code());
        }
    };

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
        Commands::Plan { action } => match action {
            PlanAction::Show { reference, answer } => commands::plan::show(reference, answer),
            PlanAction::Init { reference } => commands::plan::init(reference),
            PlanAction::Answer { reference, text } => commands::plan::answer(reference, text),
            PlanAction::Path { reference } => commands::plan::path(reference),
            PlanAction::Research { reference, dry_run } => {
                commands::plan::research(reference, dry_run)
            }
            PlanAction::Trash { reference } => commands::plan::trash(reference),
        },
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
