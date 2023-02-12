import { loadNiftyNative } from "../native.js";

export type PublishOptions = {
    cwd?: string;
    dryRun?: boolean;
    tag?: string;
    access?: string;
};

export type PublishReport = {
    root: string;
    order: string[];
    published: string[];
    skipped: string[];
};

export async function runPublish(argv: string[]): Promise<void> {
    const options = parsePublishArgs(argv);
    const native = loadNiftyNative();
    const report = native.publisher["publish-workspace"]({
        cwd: options.cwd,
        dryRun: options.dryRun,
        tag: options.tag,
        access: options.access,
    });
    printReport(report, options.dryRun);
}

function parsePublishArgs(argv: string[]): PublishOptions {
    let cwd: string | undefined;
    let dryRun = false;
    let tag: string | undefined;
    let access = "public";

    for (let i = 0; i < argv.length; i += 1) {
        const arg = argv[i];
        if (arg === "--dry-run") {
            dryRun = true;
        } else if (arg === "-C" || arg === "--cwd") {
            cwd = argv[++i];
        } else if (arg === "--tag") {
            tag = argv[++i];
        } else if (arg === "--access") {
            access = argv[++i];
        } else if (arg === "-h" || arg === "--help") {
            printPublishHelp();
            process.exit(0);
        } else {
            throw new Error(`unknown publish argument: ${arg}`);
        }
    }

    return { cwd, dryRun, tag, access };
}

function printReport(report: PublishReport, dryRun?: boolean): void {
    const prefix = dryRun ? "would publish" : "published";
    console.log(`${prefix} workspace ${report.root}`);
    if (report.skipped.length > 0) {
        console.log(`skipped private packages: ${report.skipped.join(", ")}`);
    }
    console.log("publish order:");
    for (const name of report.order) {
        const marker = report.published.includes(name) ? prefix : "pending";
        console.log(`  ${marker}: ${name}`);
    }
}

function printPublishHelp(): void {
    console.log(`nifty publish — publish workspace npm packages in dependency order

Usage:
  nifty publish
  nifty publish --dry-run
  nifty publish --tag next
  nifty publish --access public
  nifty publish -C <cwd>
`);
}
