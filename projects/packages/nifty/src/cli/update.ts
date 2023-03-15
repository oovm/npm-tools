import { spawnSync } from "node:child_process";
import { stdin as input, stdout as output } from "node:process";
import { createInterface } from "node:readline/promises";
import { dirname } from "node:path";

import { detectProjectLayout, discoverPackageDirs } from "../config/detectProject.js";

type UpdateOptions = {
    interactive: boolean;
    cwd?: string;
};

type CargoUpgrade = {
    crate: string;
    from: string;
    to: string;
};

type NpmOutdated = {
    name: string;
    current: string;
    wanted: string;
    latest: string;
};

export async function runUpdate(argv: string[]): Promise<void> {
    await updateWorkspace(parseUpdateArgs(argv));
}

export async function updateWorkspace(options: UpdateOptions): Promise<void> {
    const cwd = options.cwd ?? process.cwd();
    const layout = detectProjectLayout(cwd);

    if (layout.kind === "unknown") {
        throw new Error(`no Cargo.toml or package.json found from ${cwd}`);
    }

    if (layout.kind === "cargo" || layout.kind === "hybrid") {
        await updateCargo(layout, options.interactive);
    }
    if (layout.kind === "npm" || layout.kind === "hybrid") {
        await updateNpm(layout, options.interactive);
    }
}

function parseUpdateArgs(argv: string[]): UpdateOptions {
    let interactive = false;
    let cwd: string | undefined;
    for (let i = 0; i < argv.length; i += 1) {
        const arg = argv[i];
        if (arg === "-i" || arg === "--interactive") {
            interactive = true;
        } else if (arg === "-C" || arg === "--cwd") {
            cwd = argv[++i];
        }
    }
    return { interactive, cwd };
}

function cargoRoot(layout: ReturnType<typeof detectProjectLayout>): string | undefined {
    return (
        layout.cargoWorkspaceRoot ??
        (layout.cargoManifest ? dirname(layout.cargoManifest) : undefined)
    );
}

async function updateCargo(layout: ReturnType<typeof detectProjectLayout>, interactive: boolean): Promise<void> {
    const root = cargoRoot(layout);
    if (!root) {
        throw new Error("cargo project not found");
    }

    const dry = spawnSync("cargo", ["upgrade", "--workspace", "--dry-run"], {
        cwd: root,
        encoding: "utf8",
    });
    const combined = `${dry.stdout ?? ""}${dry.stderr ?? ""}`;
    const upgrades = parseCargoUpgrades(combined);
    if (dry.status !== 0 && !combined.includes("no upgrades found") && upgrades.length === 0) {
        throw new Error(`cargo upgrade --dry-run failed:\n${combined}`);
    }

    if (upgrades.length === 0) {
        console.log("cargo: dependencies already up to date");
        return;
    }

    if (interactive) {
        const selected = await pickItems(
            "Select cargo upgrades",
            upgrades.map((item) => `${item.crate} ${item.from} -> ${item.to}`),
        );
        for (const index of selected) {
            const item = upgrades[index];
            run("cargo", ["upgrade", "--workspace", "-p", item.crate], root);
        }
    } else {
        run("cargo", ["upgrade", "--workspace"], root);
    }
}

async function updateNpm(layout: ReturnType<typeof detectProjectLayout>, interactive: boolean): Promise<void> {
    const packages = discoverPackageDirs(layout);
    if (packages.length === 0) {
        console.log("npm: no package.json directories found");
        return;
    }
    for (const packageDir of packages) {
        await updateNpmDir(packageDir, interactive);
    }
}

async function updateNpmDir(packageDir: string, interactive: boolean): Promise<void> {
    const outdated = listNpmOutdated(packageDir);
    if (outdated.length === 0) {
        console.log(`npm (${packageDir}): dependencies already up to date`);
        return;
    }

    if (interactive) {
        const selected = await pickItems(
            `Select npm upgrades (${packageDir})`,
            outdated.map((item) => `${item.name} ${item.current} -> ${item.wanted}`),
        );
        for (const index of selected) {
            const item = outdated[index];
            run("npm", ["install", `${item.name}@${item.wanted}`], packageDir);
        }
    } else {
        run("npm", ["update"], packageDir);
    }
}

function parseCargoUpgrades(text: string): CargoUpgrade[] {
    const upgrades: CargoUpgrade[] = [];
    for (const line of text.split(/\r?\n/)) {
        const match = line.match(/^\s*(\S+)\s+v(\S+)\s+->\s+v(\S+)/);
        if (match) {
            upgrades.push({ crate: match[1], from: match[2], to: match[3] });
        }
    }
    return upgrades;
}

function listNpmOutdated(packageDir: string): NpmOutdated[] {
    const result = spawnSync("npm", ["outdated", "--json"], { cwd: packageDir, encoding: "utf8" });
    if (!result.stdout?.trim()) {
        return [];
    }
    const parsed = JSON.parse(result.stdout) as Record<
        string,
        { current: string; wanted: string; latest: string }
    >;
    return Object.entries(parsed).map(([name, entry]) => ({
        name,
        current: entry.current,
        wanted: entry.wanted,
        latest: entry.latest,
    }));
}

async function pickItems(prompt: string, labels: string[]): Promise<number[]> {
    console.log(prompt);
    for (const [index, label] of labels.entries()) {
        console.log(`  [${index}] ${label}`);
    }
    const rl = createInterface({ input, output });
    const answer = await rl.question("Indexes (comma-separated, empty = all): ");
    rl.close();
    const trimmed = answer.trim();
    if (!trimmed) {
        return labels.map((_, index) => index);
    }
    return trimmed
        .split(",")
        .map((part) => Number.parseInt(part.trim(), 10))
        .filter((index) => Number.isInteger(index) && index >= 0 && index < labels.length);
}

function run(command: string, args: string[], cwd: string): void {
    const result = spawnSync(command, args, { cwd, stdio: "inherit" });
    if (result.status !== 0) {
        throw new Error(`${command} ${args.join(" ")} failed`);
    }
}
