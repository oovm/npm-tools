export type AuthCliOptions = {
    otp?: string;
    totpSecret?: string;
    token?: string;
    npm: string;
};

export function authPayload(options: AuthCliOptions): {
    otp?: string;
    totpSecret?: string;
    token?: string;
    npm: string;
} {
    return {
        ...(options.otp !== undefined ? { otp: options.otp } : {}),
        ...(options.totpSecret !== undefined ? { totpSecret: options.totpSecret } : {}),
        ...(options.token !== undefined ? { token: options.token } : {}),
        npm: options.npm,
    };
}
