import Hopr from "@hoprnet/hopr-core"

export const getInfo = async ({ node }: { node: Hopr }) => {
    try {
        const { network, hoprTokenAddress, hoprChannelsAddress, channelClosureSecs } = node.smartContractInfo()

        return {
            amouncedAddress: (await node.getAnnouncedAddresses()).map((ma) => ma.toString()),
            listeningAddress: node.getListeningAddresses().map((ma) => ma.toString()),
            network: network,
            hoprToken: hoprTokenAddress,
            hoprChannels: hoprChannelsAddress,
            channelClosurePeriod: Math.ceil(channelClosureSecs / 60),
        }
    } catch (error) {
        return new Error("failure")
    }
}