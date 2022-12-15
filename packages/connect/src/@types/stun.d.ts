declare module 'stun' {
  declare const kMessageType: unique symbol = Symbol.for('kMessageType')
  declare const kTransactionId: unique symbol = Symbol.for('kTransactionId')
  declare const kCookie: unique symbol = Symbol.for('kCookie')
  declare const kAttributes: unique symbol = Symbol.for('kAttributes')

  type StunAttribute = {
    value: Buffer | number | AddressType
    type: number
  }

  type AddressType = {
    address: string
    port: number
    family: 'IPv4' | 'IPv6'
  }

  type ErrorType = {
    code: number
    reason?: string
  }

  declare interface StunRequest extends StunMessage {
    /**
     * Set message type.
     */
    setType(type: number): void

    /**
     * Set `transaction` field for cuurent message.
     * @param {Buffer} transactionId The value of `transaction` field.
     * @returns {boolean} Was the operation successful or not.
     */
    setTransactionId(transactionId: Buffer): boolean

    /**
     * Add an attribute for the message.
     * @param type Attribute type.
     * @param Values of an attribute.
     * @returns Return `false` if attribute already exist, otherwise return `true`.
     */
    addAttribute(type: number, ...args: any[]): StunAttribute | undefined

    /**
     * Remove attribute from current message.
     * @param type - Attribute type.
     * @returns The result of an operation.
     */
    removeAttribute(type: number): boolean

    /**
     * Add MAPPED_ADDRESS attribute.
     */
    addAddress(address: string, port: number): boolean

    /**
     * Add ALTERNATE-SERVER attribute.
     */
    addAlternateServer(address: string, port: number): boolean

    /**
     * Add XOR_MAPPED_ADDRESS attribute.
     */
    addXorAddress(address: string, port: number): boolean

    /**
     * Add USERNAME attribute.
     */
    addUsername(username: string | Buffer): boolean

    /**
     * Add ERROR-CODE attribute.
     * @param {number} code
     * @param {string} [reason]
     * @returns {boolean}
     */
    addError(code: number, reason?: string): boolean

    /**
     * Add REALM attribute.
     * @param {string} realm
     * @returns {boolean}
     */
    addRealm(realm: string): boolean

    /**
     * Add NONCE attribute.
     */
    addNonce(nonce: string): boolean

    /**
     * Add SOFTWARE attribute.
     */
    addSoftware(software: string): boolean

    /**
     * Add UNKNOWN-ATTRIBUTES attribute.
     * @param attributes List of an unknown attributes.
     * @returns
     */
    addUnknownAttributes(attributes: number[]): boolean

    /**
     * Adds a MESSAGE-INTEGRITY attribute that is valid for the current message.
     * @param key Secret hmac key.
     * @returns The result of an operation.
     */
    addMessageIntegritye(key: string): boolean

    /**
     * Adds a FINGERPRINT attribute that is valid for the current message.
     */
    addFingerprint(): boolean

    /**
     * Add PRIORITY attribute.
     */
    addPriority(priority: number): boolean

    /**
     * Add USE-CANDIDATE attribute.
     */
    addUseCandidate(): boolean

    /**
     * Add ICE-CONTROLLED attribute.
     */
    addIceControlled(tiebreaker: Buffer): boolean

    /**
   * Add ICE-CONTROLLING attribute.

   */
    addIceControlling(tiebreaker: Buffer): boolean

    /**
     * Convert current message to the Buffer.
     */
    write(encodeStream: Object): boolean

    /**
     * Convert current message to the Buffer.
     */
    toBuffer(): Buffer
  }

  declare interface StunResponse extends StunMessage {
    /**
     * Get MAPPED_ADDRESS attribute.
     */
    getAddress(): AddressType
    /**
     * Get XOR_MAPPED_ADDRESS attribute.
     */
    getXorAddress(): AddressType
    /**
     * Get ALTERNATE-SERVER attribute.
     */
    getAlternateServer(): AddressType
    /**
     * Get USERNAME attribute.
     */
    getUsername(): string
    /**
     * Get ERROR_CODE attribute.
     */
    getError(): ErrorType
    /**
     * Get REALM attribute.
     * @returns {string}
     */
    getRealm(): string
    /**
     * Get NONCE attribute.
     */
    getNonce(): string
    /**
     * Get SOFTWARE attribute.
     */
    getSoftware(): string

    /**
     * Get UNKNOWN_ATTRIBUTES attribute.
     */
    getUnknownAttributes(): number[]
    /**
     * Get MESSAGE_INTEGRITY attribute.
     */
    getMessageIntegrity(): Buffer

    /**
     * Get FINGERPRINT attribute.
     */
    getFingerprint(): number
    /**
     * Get PRIORITY attribute.
     */
    getPriority(): number

    /**
     * Get ICE_CONTROLLED attribute.
     */
    getIceControlled(): Buffer

    /**
     * Get ICE_CONTROLLING attribute.
     */
    getIceControlling(): Buffer
  }

  declare interface StunMessage extends AsyncIterable<StunAttribute> {
    [kMessageType]: number
    [kTransactionId]: Buffer
    [kCookie]: 0x2112a442 | number
    [kAttributes]: StunAttribute[]

    new (): StunMessage
    readonly type: number
    readonly transactionId: Buffer
    readonly count: number
    isLegacy(): boolean
    getAttribute(type: number): StunAttribute | undefined
    hasAttribute(type: number): boolean
  }
  declare const constants: {
    STUN_BINDING_REQUEST: number
    STUN_ATTR_RESPONSE_PORT: number
    STUN_BINDING_RESPONSE: number
    STUN_ATTR_MAPPED_ADDRESS: number
    STUN_ATTR_XOR_MAPPED_ADDRESS: number
  }

  declare function createTransaction(): Buffer

  declare function createMessage(type: number, transactionId?: Buffer): StunRequest

  declare function encode(message: StunMessage): Buffer

  declare function decode(message: Buffer): StunResponse

  declare function validateFingerprint(message: StunMessage): boolean

  declare function validateMessageIntegrity(message: StunMessage, password: string): boolean
}
