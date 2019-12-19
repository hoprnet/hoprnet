import { randomBytes } from 'crypto'
import BN from 'bn.js'

/**
 * @param start
 * @param end
 * @returns random number between @param start and @param end
 */
export default function randomInteger(start: number, end: number): number {
    if (start >= end) {
        throw Error(`Invalid interval. 'end' must be strictly greater than 'start'.`)
    }

    if (!end) {
        end = start
        start = 0
    }

    if (start + 1 == end) return start

    const byteAmount = Math.max(Math.ceil(Math.log2(end - start)) / 8, 1)

    return new BN(randomBytes(byteAmount))
        .umod(new BN(end))
        .addn(start)
        .toNumber()
}
