import { HoprOptions } from '..';
declare class StunServer {
    private server;
    constructor(options: HoprOptions);
    start(): Promise<void>;
    stop(): Promise<void>;
}
export { StunServer };
