import AbstractCommand from './abstractCommand';
export default class ListCommands implements AbstractCommand {
    execute(): void;
    complete(line: string, cb: (err: Error | undefined, hits: [string[], string]) => void): void;
}
