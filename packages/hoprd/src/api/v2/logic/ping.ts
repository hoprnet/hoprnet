import Hopr from "@hoprnet/hopr-core"
import { PeerId } from "libp2p/src/metrics"
import { checkPeerIdInput } from "../../../commands/utils"
import { APIv2State } from "../../v2"

export const ping = async ({ node, state, peerId }: { node: Hopr, state: APIv2State, peerId: string }) => {
    let validPeerId: PeerId
    try {
        validPeerId = checkPeerIdInput(peerId, state as any)
    } catch (err) {
        return new Error('invalidPeerId')
    }

    let pingResult: Awaited<ReturnType<Hopr['ping']>>
    let error: any

    try {
        pingResult = await node.ping(validPeerId)
    } catch (err) {
        error = err
    }

    if (pingResult.latency >= 0) {
        return { latency: pingResult.latency }
    }

    if (error && error.message) {
        return new Error("failure")
    }
    return new Error("timeout")
}
