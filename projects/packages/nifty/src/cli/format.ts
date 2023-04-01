import { loadNiftyNative } from "../native.js";

export type FormatOptions = {
    check?: boolean;
    cwd?: string;
};

export async function formatWorkspace(options: FormatOptions): Promise<number> {
    const native = loadNiftyNative();
    const report = native.formatter.run({
        cwd: options.cwd,
        check: options.check,
    });

    if (report.errors.length > 0) {
        for (const message of report.errors) {
            console.error(`error: ${message}`);
        }
    }

    if (report.formatted === 0 && report.unchanged > 0 && report.errors.length === 0) {
        console.log("format: no changes needed");
    } else if (report.errors.length === 0) {
        console.log(`format: ${report.formatted} file(s) updated, ${report.unchanged} unchanged`);
    }

    return report.errors.length > 0 ? 1 : 0;
}
