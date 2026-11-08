export type TrustReport = {
    root: string;
    configured: string[];
    skipped: string[];
    failed: string[];
};

export function printTrustReport(report: TrustReport): void {
    console.log(`trust workspace ${report.root}`);
    console.log(`  configured: ${report.configured.length}`);
    for (const name of report.configured) {
        console.log(`    + ${name}`);
    }
    console.log(`  skipped: ${report.skipped.length}`);
    for (const name of report.skipped) {
        console.log(`    = ${name}`);
    }
    if (report.failed.length > 0) {
        console.log(`  failed: ${report.failed.length}`);
        for (const name of report.failed) {
            console.log(`    ! ${name}`);
        }
    }
}
