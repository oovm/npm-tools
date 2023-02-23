import { loadNiftyNative } from "../native.js";
import { authPayload, parseAuthArgs } from "./authArgs.js";

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
        ...authPayload(options.auth),
    });
    printReport(report, options.dryRun);
}

function parsePublishArgs(argv: string[]) {
    let cwd: string | undefined;
    let dryRun = false;
    let tag: string | undefined;
    let access = "public";
    const auth = parseAuthArgs(argv);

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
        } else if (arg.startsWith("-")) {
            continue;
        }
    }

    return { cwd, dryRun, tag, access, auth };
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

OTP/TOTP is generated in nifty-publisher (set NPM_TOTP_SECRET in .env.placeholder.local).

Usage:
  nifty publish
  nifty publish --dry-run
  nifty publish --tag next
  nifty publish --access public
  nifty publish --otp 123456
  nifty publish -C <cwd>
`);
}
