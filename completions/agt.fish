# Fish completions for agt — agent-native todo/project management
#
# Install: copy to ~/.config/fish/completions/agt.fish
# Or:      make completions

# Disable file completions by default (agt rarely takes file paths)
complete -c agt -f

# ---------------------------------------------------------------------------
# Helper functions for dynamic completions
# ---------------------------------------------------------------------------

function __agt_todo_refs
    # Outputs "PREFIX-N\t[icon] title  P:X D:X" matching agt list style
    set -l prefix (agt config 2>/dev/null | string match -r 'Prefix:\s+(\S+)' | tail -1)
    set -l json (agt list --all --json 2>/dev/null)
    test -z "$json"; and return
    printf '%s\n' $json | command python3 -c "
import sys, json
data = json.load(sys.stdin)
prefix = '$prefix'
icons = {
    'none': '[ ]', 'todo': '[ ]', 'queued': '[~]',
    'in_progress': '[*]', 'completed': '[x]', 'archived': '[-]',
    'wont_do': '[-]', 'needs_elaboration': '[?]',
}
pri_short = {'low': 'LOW', 'medium': 'MED', 'high': 'HIGH', 'urgent': 'URGENT'}
dif_short = {'easy': 'EASY', 'medium': 'MEDIUM', 'hard': 'HARD'}
for t in data:
    ref = f'{prefix}-{t[\"number\"]}' if prefix else str(t['number'])
    icon = icons.get(t.get('status', 'none'), '[ ]')
    title = t.get('title', '')[:50].replace('\t', ' ')
    tags = ''
    p = pri_short.get(t.get('priority', ''), '')
    d = dif_short.get(t.get('difficulty', ''), '')
    if p: tags += f' P:{p}'
    if d: tags += f' D:{d}'
    assignee = t.get('assignee')
    if assignee:
        # Find member name from assignee ID — we just show @assignee
        tags += ' @'
    print(f'{ref}{icon}\t{title}{tags}')
" 2>/dev/null
end

function __agt_members
    agt member list --json 2>/dev/null | command python3 -c "
import sys, json
for m in json.load(sys.stdin):
    name = m.get('name', '')
    role = m.get('role', '')
    print(f'{name}\t{role}')
" 2>/dev/null
end

function __agt_needs_subcommand
    set -l cmd (commandline -opc)
    test (count $cmd) -eq 1
end

function __agt_using_subcommand
    set -l cmd (commandline -opc)
    test (count $cmd) -ge 2; and test "$cmd[2]" = "$argv[1]"
end

function __agt_using_subsubcommand
    set -l cmd (commandline -opc)
    test (count $cmd) -ge 3; and test "$cmd[2]" = "$argv[1]"; and test "$cmd[3]" = "$argv[2]"
end

# Count positional (non-flag) args after the subcommand
# Usage: __agt_positional_count assign → number of positional args after "assign"
function __agt_positional_count
    set -l cmd (commandline -opc)
    set -l count 0
    set -l past_sub false
    for i in (seq 2 (count $cmd))
        if test "$past_sub" = false
            if test "$cmd[$i]" = "$argv[1]"
                set past_sub true
            end
            continue
        end
        # Skip flags and their values
        if string match -q -- '--*' "$cmd[$i]"
            continue
        end
        set count (math $count + 1)
    end
    echo $count
end

# Check if subcommand needs its Nth positional arg (0-indexed)
# Usage: __agt_needs_positional assign 0  → true when no positional yet
function __agt_needs_positional
    __agt_using_subcommand $argv[1]; and test (__agt_positional_count $argv[1]) -eq $argv[2]
end

# ---------------------------------------------------------------------------
# Top-level subcommands
# ---------------------------------------------------------------------------

complete -c agt -n __agt_needs_subcommand -a init    -d "Initialize a new project"
complete -c agt -n __agt_needs_subcommand -a add     -d "Add a new todo"
complete -c agt -n __agt_needs_subcommand -a list    -d "List todos"
complete -c agt -n __agt_needs_subcommand -a show    -d "Show a single todo"
complete -c agt -n __agt_needs_subcommand -a edit    -d "Edit a todo"
complete -c agt -n __agt_needs_subcommand -a update  -d "Edit a todo (alias)"
complete -c agt -n __agt_needs_subcommand -a delete  -d "Delete a todo"
complete -c agt -n __agt_needs_subcommand -a del     -d "Delete a todo (alias)"
complete -c agt -n __agt_needs_subcommand -a assign  -d "Assign a todo to a member"
complete -c agt -n __agt_needs_subcommand -a unassign -d "Unassign a todo"
complete -c agt -n __agt_needs_subcommand -a comment -d "Add a comment to a todo"
complete -c agt -n __agt_needs_subcommand -a branch  -d "Create worktree + branch for a todo"
complete -c agt -n __agt_needs_subcommand -a unbranch -d "Remove worktree + branch for a todo"
complete -c agt -n __agt_needs_subcommand -a run     -d "Run a coding agent on a todo"
complete -c agt -n __agt_needs_subcommand -a poll    -d "Poll for queued todos and dispatch"
complete -c agt -n __agt_needs_subcommand -a queue   -d "Set todo status to queued"
complete -c agt -n __agt_needs_subcommand -a runs    -d "List active agent runs"
complete -c agt -n __agt_needs_subcommand -a plan    -d "Manage plan/research files"
complete -c agt -n __agt_needs_subcommand -a member  -d "Manage project members"
complete -c agt -n __agt_needs_subcommand -a config  -d "Show or update project config"
complete -c agt -n __agt_needs_subcommand -a serve   -d "Start the web dashboard server"
complete -c agt -n __agt_needs_subcommand -a inbox   -d "Manage the freeform inbox"
complete -c agt -n __agt_needs_subcommand -a commit  -d "Commit .todo/ changes"
complete -c agt -n __agt_needs_subcommand -a log     -d "Show audit log"

# ---------------------------------------------------------------------------
# Dynamic: todo reference completions (for commands that take a reference)
# ---------------------------------------------------------------------------

# These commands take a todo ref as their first positional arg
for subcmd in show edit update delete del assign unassign comment branch unbranch run queue
    complete -c agt -n "__agt_needs_positional $subcmd 0" -a "(__agt_todo_refs)"
end

# ---------------------------------------------------------------------------
# Dynamic: member name completions
# ---------------------------------------------------------------------------

# assign: second positional = member name
complete -c agt -n "__agt_needs_positional assign 1" -a "(__agt_members)"

# ---------------------------------------------------------------------------
# add
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand add" -l priority   -d "Priority" -r -a "none low medium high urgent"
complete -c agt -n "__agt_using_subcommand add" -l status     -d "Status" -r -a "none todo queued in_progress completed archived wont_do needs_elaboration"
complete -c agt -n "__agt_using_subcommand add" -l difficulty  -d "Difficulty" -r -a "none easy medium hard"
complete -c agt -n "__agt_using_subcommand add" -l assignee   -d "Assignee" -r -a "(__agt_members)"
complete -c agt -n "__agt_using_subcommand add" -l description -d "Description" -r
complete -c agt -n "__agt_using_subcommand add" -l labels     -d "Labels (comma-separated)" -r -a "bug new_feature feature_plus"
complete -c agt -n "__agt_using_subcommand add" -l json       -d "Output as JSON"

# ---------------------------------------------------------------------------
# list
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand list" -l status     -d "Filter by status" -r -a "none todo queued in_progress completed archived wont_do needs_elaboration"
complete -c agt -n "__agt_using_subcommand list" -l assignee   -d "Filter by assignee" -r -a "(__agt_members)"
complete -c agt -n "__agt_using_subcommand list" -l priority   -d "Filter by priority" -r -a "none low medium high urgent"
complete -c agt -n "__agt_using_subcommand list" -l difficulty  -d "Filter by difficulty" -r -a "none easy medium hard"
complete -c agt -n "__agt_using_subcommand list" -l search     -d "Search title and description" -r
complete -c agt -n "__agt_using_subcommand list" -l all        -d "Include all todos"
complete -c agt -n "__agt_using_subcommand list" -l archived   -d "Show only archived todos"
complete -c agt -n "__agt_using_subcommand list" -l rank       -d "Sort by actionability"
complete -c agt -n "__agt_using_subcommand list" -l json       -d "Output as JSON"

# ---------------------------------------------------------------------------
# show
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand show" -l json -d "Output as JSON"

# ---------------------------------------------------------------------------
# edit
# ---------------------------------------------------------------------------

for editcmd in edit update
    complete -c agt -n "__agt_using_subcommand $editcmd" -l title      -d "New title" -r
    complete -c agt -n "__agt_using_subcommand $editcmd" -l status     -d "New status" -r -a "none todo queued in_progress completed archived wont_do needs_elaboration"
    complete -c agt -n "__agt_using_subcommand $editcmd" -l priority   -d "New priority" -r -a "none low medium high urgent"
    complete -c agt -n "__agt_using_subcommand $editcmd" -l difficulty  -d "New difficulty" -r -a "none easy medium hard"
    complete -c agt -n "__agt_using_subcommand $editcmd" -l description -d "New description" -r
    complete -c agt -n "__agt_using_subcommand $editcmd" -l labels     -d "Labels" -r -a "bug new_feature feature_plus"
    complete -c agt -n "__agt_using_subcommand $editcmd" -l json       -d "Output as JSON"
end

# ---------------------------------------------------------------------------
# delete
# ---------------------------------------------------------------------------

for delcmd in delete del
    complete -c agt -n "__agt_using_subcommand $delcmd" -l json -d "Output as JSON"
end

# ---------------------------------------------------------------------------
# assign
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand assign" -l json -d "Output as JSON"

# ---------------------------------------------------------------------------
# unassign
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand unassign" -l json -d "Output as JSON"

# ---------------------------------------------------------------------------
# comment
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand comment" -l json -d "Output as JSON"

# ---------------------------------------------------------------------------
# branch / unbranch
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand branch" -l json -d "Output as JSON"
complete -c agt -n "__agt_using_subcommand unbranch" -l keep-branch -d "Keep the git branch"

# ---------------------------------------------------------------------------
# run
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand run" -l budget  -d "Max budget in USD" -r
complete -c agt -n "__agt_using_subcommand run" -l dry-run -d "Print prompt and exit"

# ---------------------------------------------------------------------------
# poll
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand poll" -l dry-run -d "Print what would be dispatched"

# ---------------------------------------------------------------------------
# runs
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand runs" -l json -d "Output as JSON"

# ---------------------------------------------------------------------------
# plan subcommands
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand plan; and not __fish_seen_subcommand_from list show init answer path research trash" -a list     -d "List all plan files"
complete -c agt -n "__agt_using_subcommand plan; and not __fish_seen_subcommand_from list show init answer path research trash" -a show     -d "Show a todo's plan"
complete -c agt -n "__agt_using_subcommand plan; and not __fish_seen_subcommand_from list show init answer path research trash" -a init     -d "Create a plan file"
complete -c agt -n "__agt_using_subcommand plan; and not __fish_seen_subcommand_from list show init answer path research trash" -a answer   -d "Append an answer to plan questions"
complete -c agt -n "__agt_using_subcommand plan; and not __fish_seen_subcommand_from list show init answer path research trash" -a path     -d "Print plan file path"
complete -c agt -n "__agt_using_subcommand plan; and not __fish_seen_subcommand_from list show init answer path research trash" -a research -d "Research and flesh out a plan"
complete -c agt -n "__agt_using_subcommand plan; and not __fish_seen_subcommand_from list show init answer path research trash" -a trash    -d "Delete a plan file"

# plan subcommands that take a todo reference
for plansubcmd in show init answer path research trash
    complete -c agt -n "__agt_using_subsubcommand plan $plansubcmd" -a "(__agt_todo_refs)"
end

complete -c agt -n "__agt_using_subsubcommand plan show"     -l answer  -s a -d "Prompt to answer questions"
complete -c agt -n "__agt_using_subsubcommand plan research" -l all     -d "Research all unplanned todos"
complete -c agt -n "__agt_using_subsubcommand plan research" -l force   -d "Overwrite existing plans"
complete -c agt -n "__agt_using_subsubcommand plan research" -l dry-run -d "Print prompt without running"

# ---------------------------------------------------------------------------
# member subcommands
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand member; and not __fish_seen_subcommand_from add list remove update" -a add    -d "Add a new member"
complete -c agt -n "__agt_using_subcommand member; and not __fish_seen_subcommand_from add list remove update" -a list   -d "List all members"
complete -c agt -n "__agt_using_subcommand member; and not __fish_seen_subcommand_from add list remove update" -a remove -d "Remove a member"
complete -c agt -n "__agt_using_subcommand member; and not __fish_seen_subcommand_from add list remove update" -a update -d "Update a member"

# member add
complete -c agt -n "__agt_using_subsubcommand member add" -l role     -d "Role" -r -a "owner member agent"
complete -c agt -n "__agt_using_subsubcommand member add" -l email    -d "Email address" -r
complete -c agt -n "__agt_using_subsubcommand member add" -l provider -d "Agent provider" -r -a "claude-code opencode custom"
complete -c agt -n "__agt_using_subsubcommand member add" -l model    -d "Agent model" -r
complete -c agt -n "__agt_using_subsubcommand member add" -l json     -d "Output as JSON"

# member list
complete -c agt -n "__agt_using_subsubcommand member list" -l json -d "Output as JSON"

# member remove — complete member names
complete -c agt -n "__agt_using_subsubcommand member remove" -a "(__agt_members)"
complete -c agt -n "__agt_using_subsubcommand member remove" -l json -d "Output as JSON"

# member update — complete member names
complete -c agt -n "__agt_using_subsubcommand member update" -a "(__agt_members)"
complete -c agt -n "__agt_using_subsubcommand member update" -l name     -d "New name" -r
complete -c agt -n "__agt_using_subsubcommand member update" -l role     -d "New role" -r -a "owner member agent"
complete -c agt -n "__agt_using_subsubcommand member update" -l email    -d "New email" -r
complete -c agt -n "__agt_using_subsubcommand member update" -l provider -d "Agent provider" -r -a "claude-code opencode custom"
complete -c agt -n "__agt_using_subsubcommand member update" -l model    -d "Agent model" -r

# ---------------------------------------------------------------------------
# config
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand config" -l name   -d "Set project name" -r
complete -c agt -n "__agt_using_subcommand config" -l prefix -d "Set issue prefix" -r

# ---------------------------------------------------------------------------
# serve
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand serve" -l port -d "Port to listen on" -r
complete -c agt -n "__agt_using_subcommand serve" -l open -d "Open in browser"

# ---------------------------------------------------------------------------
# inbox
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand inbox" -a "show append clear process" -d "Action"

# ---------------------------------------------------------------------------
# commit
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand commit" -l push    -d "Push to remote after committing"
complete -c agt -n "__agt_using_subcommand commit" -l message -s m -d "Custom commit message" -r

# ---------------------------------------------------------------------------
# log
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand log" -l limit -s n -d "Max entries" -r
complete -c agt -n "__agt_using_subcommand log" -l json      -d "Output as JSON"

# ---------------------------------------------------------------------------
# init
# ---------------------------------------------------------------------------

complete -c agt -n "__agt_using_subcommand init" -l name   -d "Project name" -r
complete -c agt -n "__agt_using_subcommand init" -l prefix -d "Project prefix" -r
