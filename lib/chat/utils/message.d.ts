/**
 * Adds the current timestamp to the message in order to measure the latency.
 * @param msg the message
 */
export declare function encodeMessage(msg: string): Uint8Array;
/**
 * Tries to decode the message and returns the message as well as
 * the measured latency.
 * @param encoded an encoded message
 */
export declare function decodeMessage(encoded: Uint8Array): {
    latency: number;
    msg: string;
};
