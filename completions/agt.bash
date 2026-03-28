# Bash completions for agt — agent-native todo/project management
#
# Install: source this file in ~/.bashrc or copy to /etc/bash_completion.d/agt
# Or:      make completions

_agt_todo_refs() {
    local prefix json
    prefix=$(agt config 2>/dev/null | sed -n 's/.*Prefix:[[:space:]]*//p')
    json=$(agt list --all --json 2>/dev/null)
    [ -z "$json" ] && return
    echo "$json" | python3 -c "
import sys, json
data = json.load(sys.stdin)
prefix = '$prefix'
icons = {
    'none': '[ ]', 'todo': '[ ]', 'paused': '[|]',
    'in_progress': '[*]', 'completed': '[x]', 'archived': '[-]',
    'wont_do': '[-]', 'needs_elaboration': '[?]',
}
for t in data:
    ref = f'{prefix}-{t[\"number\"]}' if prefix else str(t['number'])
    icon = icons.get(t.get('status', 'none'), '[ ]')
    print(f'{ref}{icon}')
" 2>/dev/null
}

_agt_members() {
    agt member list --json 2>/dev/null | python3 -c "
import sys, json
for m in json.load(sys.stdin):
    print(m.get('name', ''))
" 2>/dev/null
}

_agt_completions() {
    local cur prev subcmd
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Subcommand at position 1
    if [ "$COMP_CWORD" -eq 1 ]; then
        COMPREPLY=($(compgen -W "init add list show edit update delete del assign unassign comment branch unbranch run poll queue runs plan member config serve inbox commit log" -- "$cur"))
        return
    fi

    subcmd="${COMP_WORDS[1]}"

    # Count positional args (non-flag) after the subcommand
    local pos_count=0
    for ((i=2; i<COMP_CWORD; i++)); do
        case "${COMP_WORDS[i]}" in
            --*) ;; # skip flags
            *)   ((pos_count++)) ;;
        esac
    done

    # Handle flag value completions first
    case "$prev" in
        --priority)
            COMPREPLY=($(compgen -W "none low medium high urgent" -- "$cur"))
            return ;;
        --status)
            COMPREPLY=($(compgen -W "none todo in_progress paused completed archived wont_do needs_elaboration" -- "$cur"))
            return ;;
        --difficulty)
            COMPREPLY=($(compgen -W "none easy medium hard" -- "$cur"))
            return ;;
        --labels)
            COMPREPLY=($(compgen -W "bug new_feature feature_plus" -- "$cur"))
            return ;;
        --assignee)
            COMPREPLY=($(compgen -W "$(_agt_members)" -- "$cur"))
            return ;;
        --role)
            COMPREPLY=($(compgen -W "owner member agent" -- "$cur"))
            return ;;
        --provider)
            COMPREPLY=($(compgen -W "claude-code opencode custom" -- "$cur"))
            return ;;
    esac

    case "$subcmd" in
        # Commands that take a todo ref as first positional
        show|edit|update|delete|del|unassign|comment|branch|unbranch|run)
            if [ "$pos_count" -eq 0 ]; then
                COMPREPLY=($(compgen -W "$(_agt_todo_refs)" -- "$cur"))
            fi
            # Per-command flags
            case "$subcmd" in
                edit|update)
                    COMPREPLY+=($(compgen -W "--title --status --priority --difficulty --description --labels --json" -- "$cur")) ;;
                show|delete|del|unassign|comment|branch)
                    COMPREPLY+=($(compgen -W "--json" -- "$cur")) ;;
                unbranch)
                    COMPREPLY+=($(compgen -W "--keep-branch" -- "$cur")) ;;
                run)
                    COMPREPLY+=($(compgen -W "--budget --dry-run" -- "$cur")) ;;
            esac
            ;;
        assign)
            if [ "$pos_count" -eq 0 ]; then
                COMPREPLY=($(compgen -W "$(_agt_todo_refs)" -- "$cur"))
            elif [ "$pos_count" -eq 1 ]; then
                COMPREPLY=($(compgen -W "$(_agt_members)" -- "$cur"))
            fi
            COMPREPLY+=($(compgen -W "--json" -- "$cur"))
            ;;
        queue)
            COMPREPLY=($(compgen -W "$(_agt_todo_refs)" -- "$cur"))
            ;;
        add)
            COMPREPLY=($(compgen -W "--priority --status --difficulty --assignee --description --labels --json" -- "$cur"))
            ;;
        list)
            COMPREPLY=($(compgen -W "--status --assignee --priority --difficulty --search --all --archived --rank --json" -- "$cur"))
            ;;
        plan)
            if [ "$COMP_CWORD" -eq 2 ]; then
                COMPREPLY=($(compgen -W "list show init answer path research trash" -- "$cur"))
            else
                local plansubcmd="${COMP_WORDS[2]}"
                case "$plansubcmd" in
                    show|init|answer|path|trash)
                        if [ "$pos_count" -eq 0 ]; then
                            COMPREPLY=($(compgen -W "$(_agt_todo_refs)" -- "$cur"))
                        fi
                        case "$plansubcmd" in
                            show) COMPREPLY+=($(compgen -W "--answer" -- "$cur")) ;;
                        esac
                        ;;
                    research)
                        COMPREPLY=($(compgen -W "$(_agt_todo_refs) --all --force --dry-run" -- "$cur"))
                        ;;
                esac
            fi
            ;;
        member)
            if [ "$COMP_CWORD" -eq 2 ]; then
                COMPREPLY=($(compgen -W "add list remove update" -- "$cur"))
            else
                local membersubcmd="${COMP_WORDS[2]}"
                case "$membersubcmd" in
                    add)
                        COMPREPLY=($(compgen -W "--role --email --provider --model --json" -- "$cur")) ;;
                    list)
                        COMPREPLY=($(compgen -W "--json" -- "$cur")) ;;
                    remove)
                        COMPREPLY=($(compgen -W "$(_agt_members) --json" -- "$cur")) ;;
                    update)
                        COMPREPLY=($(compgen -W "$(_agt_members) --name --role --email --provider --model" -- "$cur")) ;;
                esac
            fi
            ;;
        config)
            COMPREPLY=($(compgen -W "--name --prefix" -- "$cur")) ;;
        serve)
            COMPREPLY=($(compgen -W "--port --open" -- "$cur")) ;;
        inbox)
            if [ "$pos_count" -eq 0 ]; then
                COMPREPLY=($(compgen -W "show append clear process" -- "$cur"))
            fi
            ;;
        commit)
            COMPREPLY=($(compgen -W "--push --message" -- "$cur")) ;;
        log)
            COMPREPLY=($(compgen -W "--limit --json" -- "$cur")) ;;
        init)
            COMPREPLY=($(compgen -W "--name --prefix" -- "$cur")) ;;
        poll)
            COMPREPLY=($(compgen -W "--dry-run" -- "$cur")) ;;
        runs)
            COMPREPLY=($(compgen -W "--json" -- "$cur")) ;;
    esac
}

complete -o default -F _agt_completions agt
