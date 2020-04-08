export declare class PRG {
    private readonly key;
    private readonly iv;
    private initialised;
    private constructor();
    static get IV_LENGTH(): number;
    static get KEY_LENGTH(): number;
    static createPRG(key: Uint8Array, iv: Uint8Array): PRG;
    digest(start: number, end: number): Uint8Array;
}
