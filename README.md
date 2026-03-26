# agent-todo-list

![alt text](image.png)

A local-first, CRDT-backed task manager for developers and AI coding agents.
CLI for agents and power users. Web dashboard for visual management.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/kaiwen-wang/agent-todo-list/main/install.sh | sh
```

This downloads the latest pre-built binary to `~/.local/bin/agt` and web assets to `~/.local/share/agt/web`. Make sure `~/.local/bin` is in your `PATH`.

### Build from source

Requires Rust and Bun.

```sh
git clone https://github.com/kaiwen-wang/agent-todo-list.git
cd agent-todo-list
make deploy
```

## Quick start

```sh
cd my-project
agt init
agt add "my first task" --priority high
agt list
agt serve        # open web dashboard at localhost:3000
```

## About

- Local-first: data lives in your repo as `.todo/data.automerge`, syncs via git
- Agent-native: CLI-first design, no interactive prompts, `--json` output
- CRDT-backed: Automerge handles merge conflicts automatically
- Inspired by [Beans](https://github.com/hmans/beans), [Backlog.md](https://github.com/MrLesk/Backlog.md), Linear, Jira

## Architecture

- CLI: Rust (`src/rust/`)
- Web: Vue 3 + Vite (`src/web/`)
- Data: Automerge CRDT
- Sync: git push/pull with custom merge driver

Run `agt --all` for full CLI reference.
