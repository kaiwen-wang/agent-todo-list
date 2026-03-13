#!/usr/bin/env bun
/**
 * agt — CLI entry point.
 * Agent-native todo/project management tool.
 */

import { Command } from "commander";
import { registerInit } from "./commands/init.js";
import { registerAdd } from "./commands/add.js";
import { registerList } from "./commands/list.js";
import { registerShow } from "./commands/show.js";
import { registerUpdate } from "./commands/update.js";
import { registerDelete } from "./commands/delete.js";
import { registerAssign } from "./commands/assign.js";
import { registerUnassign } from "./commands/unassign.js";
import { registerComment } from "./commands/comment.js";
import { registerBranch } from "./commands/branch.js";
import { registerMember } from "./commands/member.js";
import { registerConfig } from "./commands/config.js";
import { registerServe } from "./commands/browser.js";
import { registerInbox } from "./commands/inbox.js";
import { registerBrain } from "./commands/brain.js";
import { registerLog } from "./commands/log.js";

const VERSION = "0.9.0";
const program = new Command();

program
	.name("agt")
	.description("Agent-native todo/project management tool")
	.version(VERSION, "-v, --version", "output the version number");

// Register all commands
registerInit(program);
registerAdd(program);
registerList(program);
registerShow(program);
registerUpdate(program);
registerDelete(program);
registerAssign(program);
registerUnassign(program);
registerComment(program);
registerBranch(program);
registerMember(program);
registerConfig(program);
registerServe(program);
registerInbox(program);
registerBrain(program);
registerLog(program);

// -h -f / --help --full / -hf: print help for every subcommand
const args = process.argv.slice(2);
const hasHelp = args.includes("--help") || args.includes("-h") || args.includes("-hf");
const hasFull = args.includes("--full") || args.includes("-f") || args.includes("-hf");
if (hasHelp && hasFull) {
	program.outputHelp();
	for (const sub of program.commands) {
		console.log("\n" + "─".repeat(60));
		sub.outputHelp();
	}
	process.exit(0);
}

program.parse();
