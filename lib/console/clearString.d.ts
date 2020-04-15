/// <reference types="node" />
import readline from 'readline';
/**
 * Takes a string that has been printed on the console and deletes
 * it line by line from the console.
 *
 * @notice Mainly used to get rid of questions printed to the console
 *
 * @param str string to delete
 * @param rl readline handle
 */
export declare function clearString(str: string, rl: readline.Interface): void;
