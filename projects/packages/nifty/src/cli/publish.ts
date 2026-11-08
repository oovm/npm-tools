export type PublishReport = {
    root: string;
    order: string[];
    published: string[];
    skipped: string[];
    skippedVersions: string[];
    staged: string[];
};

export function printPublishReport(report: PublishReport, dryRun?: boolean): void {
    const prefix = dryRun ? "would publish" : "published";
    console.log(`${prefix} workspace ${report.root}`);
    if (report.skipped.length > 0) {
        console.log(`skipped private packages: ${report.skipped.join(", ")}`);
    }
    if (report.skippedVersions.length > 0) {
        console.log(`skipped already published: ${report.skippedVersions.join(", ")}`);
    }
    if (report.staged.length > 0) {
        console.log(`staged but not live: ${report.staged.join(", ")}`);
    }
    console.log("publish order:");
    for (const name of report.order) {
        const marker = report.published.includes(name) ? prefix : "pending";
        console.log(`  ${marker}: ${name}`);
    }
}
