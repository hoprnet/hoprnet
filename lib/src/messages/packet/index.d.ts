import PeerId from 'peer-id';
import { Header } from './header';
import { Challenge } from './challenge';
import Message from './message';
import { LevelUp } from 'levelup';
import Hopr from '../../';
import HoprCoreConnector, { Types } from '@hoprnet/hopr-core-connector-interface';
/**
 * Encapsulates the internal representation of a packet
 */
export declare class Packet<Chain extends HoprCoreConnector> extends Uint8Array {
    private _targetPeerId?;
    private _senderPeerId?;
    private _header?;
    private _ticket?;
    private _challenge?;
    private _message?;
    private node;
    constructor(node: Hopr<Chain>, arr?: {
        bytes: ArrayBuffer;
        offset: number;
    }, struct?: {
        header: Header<Chain>;
        ticket: Types.SignedTicket<Types.Ticket, Types.Signature>;
        challenge: Challenge<Chain>;
        message: Message;
    });
    subarray(begin?: number, end?: number): Uint8Array;
    get headerOffset(): number;
    get header(): Header<Chain>;
    get ticketOffset(): number;
    get ticket(): Types.SignedTicket<Types.Ticket, Types.Signature>;
    get challengeOffset(): number;
    get challenge(): Challenge<Chain>;
    get messageOffset(): number;
    get message(): Message;
    static SIZE<Chain extends HoprCoreConnector>(hoprCoreConnector: Chain): number;
    /**
     * Creates a new packet.
     *
     * @param node the node itself
     * @param msg the message that is sent through the network
     * @param path array of peerId that determines the route that
     * the packet takes
     */
    static create<Chain extends HoprCoreConnector>(node: Hopr<Chain>, msg: Uint8Array, path: PeerId[]): Promise<Packet<Chain>>;
    /**
     * Tries to get a previous transaction from the database. If there's no such one,
     * listen to the channel opening event for some time and throw an error if the
     * was not opened within `OPENING_TIMEOUT` ms.
     *
     * @param channelId ID of the channel
     */
    getPreviousTransaction(channelId: Uint8Array, state: any): Promise<void>;
    /**
     * Checks the packet and transforms it such that it can be send to the next node.
     *
     * @param node the node itself
     */
    forwardTransform(): Promise<{
        receivedChallenge: Challenge<Chain>;
        ticketKey: Uint8Array;
    }>;
    /**
     * Prepares the delivery of the packet.
     *
     * @param node the node itself
     * @param state current off-chain state
     * @param newState future off-chain state
     * @param nextNode the ID of the payment channel
     */
    prepareDelivery(state: any, newState: any, nextNode: any): Promise<void>;
    /**
     * Prepares the packet in order to forward it to the next node.
     *
     * @param node the node itself
     * @param state current off-chain state
     * @param newState future off-chain state
     * @param channelId the ID of the payment channel
     * @param target peer Id of the next node
     */
    prepareForward(state: any, newState: any, target: PeerId): Promise<void>;
    /**
     * Computes the peerId of the next downstream node and caches it for later use.
     */
    getTargetPeerId(): Promise<PeerId>;
    /**
     * Computes the peerId if the preceeding node and caches it for later use.
     */
    getSenderPeerId(): Promise<PeerId>;
    /**
     * Checks whether the packet has already been seen.
     */
    testAndSetTag(db: LevelUp): Promise<boolean>;
}
