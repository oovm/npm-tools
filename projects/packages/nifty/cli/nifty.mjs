#!/usr/bin/env node

const { runCli } = await import("../dist/cli/main.js");

process.exit(await runCli(process.argv.slice(2)));
