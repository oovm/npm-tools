import { loadNiftyNative } from "../native.js";

export type UpdateOptions = {
    interactive: boolean;
    cwd?: string;
};

export async function updateWorkspace(options: UpdateOptions): Promise<void> {
    const native = loadNiftyNative();
    native.updater.run({
        cwd: options.cwd,
        interactive: options.interactive,
    });
}
