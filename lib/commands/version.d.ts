import type AbstractCommand from './abstractCommand';
export default class Version implements AbstractCommand {
    #private;
    execute(): Promise<void>;
    complete(): void;
}
