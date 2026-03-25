/**
 * Brain — AI agent that processes the inbox into structured tasks.
 *
 * Spawns the `claude` CLI with a structured prompt containing:
 * - The inbox text to process
 * - The full `agt` CLI reference so Claude knows what commands are available
 * - Current project context (members, existing tasks) to avoid duplicates
 *
 * Events are emitted via a callback so callers (server/CLI) can stream them.
 */

import { join } from "node:path";
import { readInbox, writeInbox, appendProcessed, type ProcessedEntry } from "./inbox.js";
import { loadDoc } from "./storage.js";
import { toJSON } from "./export.js";
import { migrateDoc, needsMigration } from "./migrate.js";

// ── Event types ─────────────────────────────────────────────────────

export type BrainEvent =
  | { type: "brain:log"; message: string }
  | { type: "brain:task"; ref: string; title: string; original: string }
  | { type: "brain:error"; message: string }
  | { type: "brain:done"; processed: number; tasks: Array<{ ref: string; title: string }> };

type EventCallback = (event: BrainEvent) => void;

// ── CLI Reference (embedded documentation for the AI agent) ─────────

const CLI_REFERENCE = `
## agt CLI Reference

You are operating inside a project managed by \`agt\` (agent-todo). Use these commands to create and manage tasks.

### Create a task
\`\`\`
agt add <title> [options]
  -d, --description <text>    Description (markdown)
  -s, --status <status>       Initial status (default: "none")
  -p, --priority <priority>   Priority level (default: "none")
  -a, --assignee <name>       Assignee name (must be an existing member)
  -l, --labels <labels>       Comma-separated labels
  --json                      Output as JSON (ALWAYS use this flag)
\`\`\`

### Valid values
- **status**: none, todo, queued, needs_elaboration, in_progress, completed, archived, wont_do
- **priority**: none, low, medium, high, urgent
- **labels**: bug, new_feature, feature_plus

### Other useful commands
\`\`\`
agt list --json                       List all tasks
agt show <ref> --json                 Show task detail (ref = PREFIX-N or just N)
agt update <ref> [options] --json     Update a task
agt comment <ref> <text> --json       Add a comment
agt assign <ref> <member> --json      Assign a task
\`\`\`

### Important
- ALWAYS use --json flag so output can be parsed
- Task refs use the format PREFIX-NUMBER (e.g. TODO-1, ISD-3)
- Run tasks via \`bun run agt <command>\`
`.trim();

// ── Prompt builder ──────────────────────────────────────────────────

function buildPrompt(inboxText: string, projectContext: string): string {
  return `You are a task management agent. Your job is to read the user's freeform inbox notes and create structured tasks from them using the \`agt\` CLI tool.

${CLI_REFERENCE}

## Current Project Context
${projectContext}

## Inbox Notes to Process
\`\`\`
${inboxText.trim()}
\`\`\`

## Instructions

1. Read through each distinct item/thought in the inbox above.
2. For EACH item, run \`bun run agt add "<title>" --json\` with appropriate options:
   - Write a clear, concise title
   - Add a description with \`-d\` if the note needs elaboration
   - Set priority with \`-p\` based on urgency cues (words like "urgent", "critical" = urgent/high; "eventually", "maybe" = low)
   - Set labels with \`-l\` if applicable (bug reports = bug, new functionality = new_feature)
   - Assign with \`-a\` if the note mentions a specific person
3. Multiple items can be created from a single note if it contains distinct tasks.
4. Do NOT create duplicate tasks if similar ones already exist in the project.
5. After creating all tasks, output a final summary line: BRAIN_DONE

Be efficient. Do not explain your reasoning. Just create the tasks.`;
}

/** Build a summary of the current project state for context. */
async function getProjectContext(dataPath: string): Promise<string> {
  let doc = await loadDoc(dataPath);
  if (!doc) return "(no project data available)";

  if (needsMigration(doc)) {
    doc = migrateDoc(doc);
  }

  const json = toJSON(doc) as {
    prefix: string;
    name: string;
    members: Array<{ name: string; role: string }>;
    todos: Array<{ ref: string; title: string; status: string }>;
  };

  const lines: string[] = [];
  lines.push(`Project: ${json.name} (prefix: ${json.prefix})`);

  if (json.members.length > 0) {
    lines.push(`Members: ${json.members.map((m) => `${m.name} (${m.role})`).join(", ")}`);
  }

  const activeTodos = json.todos.filter((t) => t.status !== "archived" && t.status !== "wont_do");
  if (activeTodos.length > 0) {
    lines.push(`\nExisting tasks (${activeTodos.length}):`);
    for (const t of activeTodos.slice(0, 30)) {
      lines.push(`  - ${t.ref}: ${t.title} [${t.status}]`);
    }
    if (activeTodos.length > 30) {
      lines.push(`  ... and ${activeTodos.length - 30} more`);
    }
  } else {
    lines.push("No existing tasks.");
  }

  return lines.join("\n");
}

/** Get all todo refs and titles from the current doc on disk. */
async function getTodoRefs(dataPath: string): Promise<Array<{ ref: string; title: string }>> {
  let doc = await loadDoc(dataPath);
  if (!doc) return [];
  if (needsMigration(doc)) doc = migrateDoc(doc);
  const json = toJSON(doc) as { todos: Array<{ ref: string; title: string }> };
  return json.todos.map((t) => ({ ref: t.ref, title: t.title }));
}

// ── Main processor ──────────────────────────────────────────────────

export async function processInbox(projectPath: string, onEvent: EventCallback): Promise<void> {
  const todoDir = join(projectPath, ".todo");
  const dataPath = join(todoDir, "data.automerge");

  // 1. Read inbox
  const inboxText = await readInbox(todoDir);
  if (!inboxText.trim()) {
    onEvent({ type: "brain:done", processed: 0, tasks: [] });
    return;
  }

  onEvent({ type: "brain:log", message: "Reading inbox..." });

  // 2. Build prompt with project context
  const context = await getProjectContext(dataPath);
  const prompt = buildPrompt(inboxText, context);

  // 3. Snapshot existing todo refs before spawning Claude
  const existingRefs = await getTodoRefs(dataPath);

  onEvent({ type: "brain:log", message: "Spawning Claude agent..." });

  // 4. Spawn claude CLI
  const proc = Bun.spawn(
    [
      "claude",
      "-p",
      prompt,
      "--verbose",
      "--output-format",
      "stream-json",
      "--allowedTools",
      "Bash(bun run agt*)",
      "--permission-mode",
      "bypassPermissions",
      "--max-budget-usd",
      "1.00",
    ],
    {
      cwd: projectPath,
      stdout: "pipe",
      stderr: "pipe",
    },
  );

  // 5. Stream stdout line-by-line, parse stream-json events
  const decoder = new TextDecoder();
  let buffer = "";

  const reader = proc.stdout.getReader();

  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });

      // Process complete lines
      const lines = buffer.split("\n");
      buffer = lines.pop() ?? "";

      for (const line of lines) {
        if (!line.trim()) continue;
        processStreamLine(line, onEvent);
      }
    }

    // Process any remaining buffer
    if (buffer.trim()) {
      processStreamLine(buffer, onEvent);
    }
  } catch (err) {
    onEvent({
      type: "brain:error",
      message: `Stream error: ${err instanceof Error ? err.message : String(err)}`,
    });
  }

  // 6. Wait for process to exit
  const exitCode = await proc.exited;

  // Check stderr for errors
  const stderrText = await new Response(proc.stderr).text();
  if (exitCode !== 0 && stderrText.trim()) {
    onEvent({
      type: "brain:error",
      message: `Claude exited with code ${exitCode}: ${stderrText.trim()}`,
    });
  }

  // 7. Detect newly created tasks by comparing doc state before/after
  const currentRefs = await getTodoRefs(dataPath);
  const existingRefSet = new Set(existingRefs.map((t) => t.ref));
  const newTasks = currentRefs.filter((t) => !existingRefSet.has(t.ref));

  // 8. Archive processed items and clear inbox
  if (newTasks.length > 0) {
    const inboxLines = inboxText
      .split("\n")
      .map((l) => l.replace(/^[-*]\s*/, "").trim())
      .filter(Boolean);

    const processedEntries: ProcessedEntry[] = [];
    for (const task of newTasks) {
      const match =
        inboxLines.find(
          (line) =>
            task.title.toLowerCase().includes(line.toLowerCase().slice(0, 20)) ||
            line.toLowerCase().includes(task.title.toLowerCase().slice(0, 20)),
        ) ?? "(auto-expanded)";

      processedEntries.push({ original: match, ref: task.ref, title: task.title });
      onEvent({ type: "brain:task", ref: task.ref, title: task.title, original: match });
    }

    await appendProcessed(todoDir, processedEntries);
    await writeInbox(todoDir, "");

    onEvent({
      type: "brain:log",
      message: `Archived ${processedEntries.length} items to TODO-PROCESSED.md and cleared inbox.`,
    });
  }

  // 9. Done
  onEvent({
    type: "brain:done",
    processed: newTasks.length,
    tasks: newTasks,
  });
}

/**
 * Parse a single line from Claude's stream-json output and emit events.
 *
 * The stream-json format outputs one JSON object per line:
 * - { type: "system", subtype: "init", ... } — Session init
 * - { type: "assistant", message: { content: [...] } } — Claude's response
 *     Content blocks can be { type: "text" } or { type: "tool_use" }
 * - { type: "result", is_error, result, ... } — Final result
 */
function processStreamLine(line: string, onEvent: EventCallback): void {
  try {
    const event = JSON.parse(line);

    switch (event.type) {
      case "assistant": {
        // Assistant events contain content blocks: text and tool_use
        const content = event.message?.content;
        if (Array.isArray(content)) {
          for (const block of content) {
            if (block.type === "text" && block.text) {
              onEvent({ type: "brain:log", message: block.text });
            } else if (block.type === "tool_use" && block.input?.command) {
              onEvent({ type: "brain:log", message: `$ ${block.input.command}` });
            }
          }
        }
        break;
      }

      case "result": {
        if (event.is_error) {
          const text = typeof event.result === "string" ? event.result : extractText(event);
          onEvent({ type: "brain:error", message: text || "Claude encountered an unknown error" });
        }
        break;
      }

      default: {
        if (event.error) {
          const msg = typeof event.error === "string" ? event.error : JSON.stringify(event.error);
          onEvent({ type: "brain:error", message: msg });
        }
        break;
      }
    }
  } catch {
    // Not valid JSON — treat as raw text output
    if (line.trim()) {
      onEvent({ type: "brain:log", message: line });
    }
  }
}

/** Extract text content from a stream-json event. */
function extractText(event: Record<string, unknown>): string {
  // Direct content field
  if (typeof event.content === "string") return event.content;
  if (Array.isArray(event.content)) {
    return event.content
      .filter((c: { type?: string }) => c.type === "text")
      .map((c: { text?: string }) => c.text ?? "")
      .join("");
  }
  // Nested message.content (assistant events wrap content inside message)
  if (event.message && typeof event.message === "object") {
    const msg = event.message as Record<string, unknown>;
    if (Array.isArray(msg.content)) {
      return msg.content
        .filter((c: { type?: string }) => c.type === "text")
        .map((c: { text?: string }) => c.text ?? "")
        .join("");
    }
    if (typeof msg.content === "string") return msg.content;
  }
  if (typeof event.message === "string") return event.message;
  // Result events put text in the result field
  if (typeof event.result === "string") return event.result;
  return "";
}

/** Dry-run mode — just return the prompt that would be sent. */
export async function dryRun(projectPath: string): Promise<string> {
  const todoDir = join(projectPath, ".todo");
  const dataPath = join(todoDir, "data.automerge");

  const inboxText = await readInbox(todoDir);
  if (!inboxText.trim()) return "(inbox is empty — nothing to process)";

  const context = await getProjectContext(dataPath);
  return buildPrompt(inboxText, context);
}
