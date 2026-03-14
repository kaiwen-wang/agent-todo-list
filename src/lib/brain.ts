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
  -s, --status <status>       Initial status (default: "todo")
  -p, --priority <priority>   Priority level (default: "none")
  -a, --assignee <name>       Assignee name (must be an existing member)
  -l, --labels <labels>       Comma-separated labels
  --json                      Output as JSON (ALWAYS use this flag)
\`\`\`

### Valid values
- **status**: none, todo, needs_elaboration, in_progress, completed, archived, wont_do
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

  onEvent({ type: "brain:log", message: "Spawning Claude agent..." });

  // 3. Spawn claude CLI
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

  // 4. Stream stdout line-by-line, parse stream-json events
  const createdTasks: Array<{ ref: string; title: string }> = [];
  const processedEntries: ProcessedEntry[] = [];
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
        processStreamLine(line, onEvent, createdTasks);
      }
    }

    // Process any remaining buffer
    if (buffer.trim()) {
      processStreamLine(buffer, onEvent, createdTasks);
    }
  } catch (err) {
    onEvent({
      type: "brain:error",
      message: `Stream error: ${err instanceof Error ? err.message : String(err)}`,
    });
  }

  // 5. Wait for process to exit
  const exitCode = await proc.exited;

  // Check stderr for errors
  const stderrText = await new Response(proc.stderr).text();
  if (exitCode !== 0 && stderrText.trim()) {
    onEvent({
      type: "brain:error",
      message: `Claude exited with code ${exitCode}: ${stderrText.trim()}`,
    });
  }

  // 6. Archive processed items and clear inbox
  if (createdTasks.length > 0) {
    // Build processed entries — match created tasks back to inbox lines
    const inboxLines = inboxText
      .split("\n")
      .map((l) => l.replace(/^[-*]\s*/, "").trim())
      .filter(Boolean);

    for (const task of createdTasks) {
      // Find the closest matching inbox line
      const match =
        inboxLines.find(
          (line) =>
            task.title.toLowerCase().includes(line.toLowerCase().slice(0, 20)) ||
            line.toLowerCase().includes(task.title.toLowerCase().slice(0, 20)),
        ) ?? "(auto-expanded)";

      processedEntries.push({
        original: match,
        ref: task.ref,
        title: task.title,
      });
    }

    await appendProcessed(todoDir, processedEntries);
    await writeInbox(todoDir, "");

    onEvent({
      type: "brain:log",
      message: `Archived ${processedEntries.length} items to TODO-PROCESSED.md and cleared inbox.`,
    });
  }

  // 7. Done
  onEvent({
    type: "brain:done",
    processed: createdTasks.length,
    tasks: createdTasks,
  });
}

/**
 * Parse a single line from Claude's stream-json output and emit events.
 *
 * The stream-json format outputs one JSON object per line with varying types:
 * - { type: "assistant", ... } — Claude's text output
 * - { type: "tool_use", ... } — Claude is running a tool
 * - { type: "tool_result", ... } — Tool execution result
 * - { type: "result", ... } — Final result
 */
function processStreamLine(
  line: string,
  onEvent: EventCallback,
  createdTasks: Array<{ ref: string; title: string }>,
): void {
  try {
    const event = JSON.parse(line);

    switch (event.type) {
      case "assistant": {
        // Claude text output — forward as log
        const text = extractText(event);
        if (text) {
          onEvent({ type: "brain:log", message: text });
        }
        break;
      }

      case "tool_use": {
        // Claude is about to run a command
        const input = event.tool_use?.input ?? event.input;
        if (input?.command) {
          onEvent({ type: "brain:log", message: `$ ${input.command}` });
        }
        break;
      }

      case "tool_result": {
        // Tool finished — check if it was an agt add that created a task
        const content = event.tool_result?.content ?? event.content;
        const text = typeof content === "string" ? content : JSON.stringify(content);

        if (text) {
          // Try to parse agt add --json output: { "ref": "ISD-5", "number": 5, "title": "..." }
          const taskMatch = text.match(/"ref"\s*:\s*"([^"]+)"/);
          const titleMatch = text.match(/"title"\s*:\s*"([^"]+)"/);
          // Also try the simpler format: { ref, number, title }
          const numberMatch = text.match(/"number"\s*:\s*(\d+)/);

          if (taskMatch || numberMatch) {
            try {
              // Try to parse the full JSON for accurate extraction
              const jsonStr = text.match(/\{[^{}]*\}/)?.[0];
              if (jsonStr) {
                const parsed = JSON.parse(jsonStr);
                if (parsed.ref || parsed.number) {
                  const ref = parsed.ref ?? `?-${parsed.number}`;
                  const title = parsed.title ?? "(untitled)";
                  createdTasks.push({ ref, title });
                  onEvent({ type: "brain:task", ref, title, original: "" });
                }
              }
            } catch {
              // If JSON parsing fails, use regex matches
              if (taskMatch?.[1]) {
                const ref = taskMatch[1];
                const title = titleMatch?.[1] ?? "(untitled)";
                createdTasks.push({ ref, title });
                onEvent({ type: "brain:task", ref, title, original: "" });
              }
            }
          }

          // Forward non-JSON output as log
          if (!text.startsWith("{")) {
            onEvent({ type: "brain:log", message: text });
          }
        }
        break;
      }

      case "result": {
        // Check if the result indicates an error (e.g., auth failure)
        if (event.is_error) {
          const text = typeof event.result === "string" ? event.result : extractText(event);
          onEvent({ type: "brain:error", message: text || "Claude encountered an unknown error" });
        } else {
          const text = extractText(event);
          if (text) {
            onEvent({ type: "brain:log", message: text });
          }
        }
        break;
      }

      default: {
        // Log unhandled event types for debugging (e.g., "system" init events)
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
