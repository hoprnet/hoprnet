import stripAnsi from 'strip-ansi'
import readline from 'readline'

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