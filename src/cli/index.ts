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
import { registerAssign } from "./commands/assign.js";
import { registerExport } from "./commands/export.js";
import { registerBrowser } from "./commands/browser.js";

const program = new Command();

program
  .name("agt")
  .description("Agent-native todo/project management tool")
  .version("0.1.0");

// Register all commands
registerInit(program);
registerAdd(program);
registerList(program);
registerShow(program);
registerUpdate(program);
registerAssign(program);
registerExport(program);
registerBrowser(program);

program.parse();
