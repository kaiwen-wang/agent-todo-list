# agent-todo-list

![alt text](image.png)

- always felt tasks should be local
- inspired by:
    - https://github.com/hmans/beans
    - https://github.com/MrLesk/Backlog.md
    - linear, jira

# architecture

- cli, vue frontend
    - automerge, lefthook, oxfmt, oxlint
    - naive ui
    - makefile
    - bun (will rewrite in Rust or go, binary is too big)

- mcps are outdated
- terminal ui often too hard to use, went for solely cli + web

# todo:

- consider integrating background AI agents that automatically run the tasks you specify
- mobile apps to view tasks