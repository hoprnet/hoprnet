/// <reference types="node" />
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
import type Hopr from '..';
import { EventEmitter } from 'events';
declare class Heartbeat<Chain extends HoprCoreConnector> extends EventEmitter {
    node: Hopr<Chain>;
    heap: string[];
    nodes: Map<string, number>;
    interval: any;
    constructor(node: Hopr<Chain>);
    private connectionListener;
    private comparator;
    checkNodes(): Promise<void>;
    start(): void;
    stop(): void;
}
export { Heartbeat };
