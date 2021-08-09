import Ganache from 'ganache-core';
declare class CustomGanache {
    private server;
    private ops;
    constructor(customOps?: Ganache.IServerOptions);
    start(): Promise<this>;
    stop(): Promise<this>;
    restart(): Promise<this>;
}
export default CustomGanache;
