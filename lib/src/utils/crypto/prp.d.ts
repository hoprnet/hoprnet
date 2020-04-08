export declare class PRP {
    private readonly k1;
    private readonly k2;
    private readonly k3;
    private readonly k4;
    private readonly iv1;
    private readonly iv2;
    private readonly iv3;
    private readonly iv4;
    private initialised;
    private constructor();
    static get KEY_LENGTH(): number;
    static get IV_LENGTH(): number;
    static get MIN_LENGTH(): number;
    static createPRP(key: Uint8Array, iv: Uint8Array): PRP;
    permutate(plaintext: Uint8Array): Uint8Array;
    inverse(ciphertext: Uint8Array): Uint8Array;
}
