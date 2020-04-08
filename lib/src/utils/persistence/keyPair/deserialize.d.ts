import PeerId from 'peer-id';
/**
 * Deserializes a serialized key pair and returns a peerId.
 *
 * @notice This method will ask for a password to decrypt the encrypted
 * private key.
 * @notice The decryption of the private key makes use of a memory-hard
 * hash function and consumes therefore a lot of memory.
 *
 * @param encryptedSerializedKeyPair the encoded and encrypted key pair
 */
export declare function deserializeKeyPair(encryptedSerializedKeyPair: Uint8Array, password: Uint8Array): Promise<PeerId>;
