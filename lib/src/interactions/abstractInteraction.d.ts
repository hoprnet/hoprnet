import type Hopr from '..';
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface';
export declare type Sink = (source: AsyncIterable<Uint8Array>) => void;
export declare type Source = AsyncIterator<Uint8Array>;
export declare type Duplex = {
    sink: Sink;
    source: Source;
};
interface AbstractInteraction<Chain extends HoprCoreConnector> {
    protocols: string[];
    node: Hopr<Chain>;
    handler(struct: {
        stream: Duplex;
    }): void;
    interact(...props: any[]): any;
}
export { AbstractInteraction };
