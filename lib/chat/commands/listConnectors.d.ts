import AbstractCommand from './abstractCommand';
export default class ListConnectors implements AbstractCommand {
    /**
     * Check which connectors are present right now.
     * @notice triggered by the CLI
     */
    execute(): Promise<void>;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void;
}
