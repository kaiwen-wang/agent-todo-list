/**
 * `agt brain` — AI agent that processes the inbox into structured tasks.
 */

import type { Command } from "commander";
import chalk from "chalk";
import { findProject } from "../../lib/project.js";
import { processInbox, dryRun, type BrainEvent } from "../../lib/brain.js";
import { error } from "../output.js";

export function registerBrain(program: Command): void {
  const brain = program
    .command("brain")
    .description("AI agent for processing inbox items into tasks");

  brain
    .command("process")
    .description("Process inbox items into tasks using Claude")
    .option("--dry-run", "Show the prompt that would be sent to Claude without executing")
    .action(async (opts: { dryRun?: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      if (opts.dryRun) {
        const prompt = await dryRun(paths.root);
        console.log(chalk.dim("--- Prompt that would be sent to Claude ---"));
        console.log(prompt);
        console.log(chalk.dim("--- End of prompt ---"));
        return;
      }

      console.log(chalk.bold("Brain processing inbox...\n"));

      await processInbox(paths.root, (event: BrainEvent) => {
        switch (event.type) {
          case "brain:log":
            console.log(chalk.dim("> ") + event.message);
            break;
          case "brain:task":
            console.log(chalk.green(`  + Created ${chalk.bold(event.ref)}: ${event.title}`));
            break;
          case "brain:error":
            console.error(chalk.red(`  ! ${event.message}`));
            break;
          case "brain:done":
            console.log("");
            if (event.processed > 0) {
              console.log(
                chalk.green.bold(
                  `Done! Created ${event.processed} task${event.processed === 1 ? "" : "s"}.`,
                ),
              );
            } else {
              console.log(chalk.yellow("No tasks were created."));
            }
            break;
        }
      });
    });
}
