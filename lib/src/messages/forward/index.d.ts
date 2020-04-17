import PeerId from 'peer-id';
declare class ForwardPacket extends Uint8Array {
    constructor(arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        destination: PeerId;
        sender: PeerId;
        payload?: Uint8Array;
    });
    subarray(begin?: number, end?: number): Uint8Array;
    get destination(): Uint8Array;
    get sender(): Uint8Array;
    get payload(): Uint8Array;
}
export { ForwardPacket };
