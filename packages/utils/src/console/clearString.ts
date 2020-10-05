import stripAnsi from 'strip-ansi'
import readline from 'readline'

/**
 * Takes a string that has been printed on the console and deletes
 * it line by line from the console.
 *
 * @notice Mainly used to get rid of questions printed to the console
 *
 * @param str string to delete
 * @param rl readline handle
 */
export function clearString(str: string, rl: readline.Interface) {
  const newLines = str.split(/\n/g)

  let lines = 0
  let stripped: string
  for (let i = 0; i < newLines.length; i++) {
    stripped = stripAnsi(newLines[i])
    if (stripped.length > process.stdout.columns) {
      lines += Math.ceil(stripped.length / process.stdout.columns)
    } else {
      lines++
    }
  }

  for (let i = 0; i < lines; i++) {
    readline.moveCursor(process.stdout, -rl.line, -1)
    readline.clearLine(process.stdout, 0)
  }
}
