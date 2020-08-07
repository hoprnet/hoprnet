import Hopr from "@hoprnet/hopr-core";
import type { HoprOptions } from "@hoprnet/hopr-core";
import type HoprCoreConnector from "@hoprnet/hopr-core-connector-interface";
import PeerInfo from "peer-info";
import PeerId from "peer-id";
import Multiaddr from "multiaddr";

let NODE: Hopr<HoprCoreConnector>;

function log(message: string) {
  console.log(message);
}

// DEFAULT VALUES FOR NOW
const network = process.env.HOPR_NETWORK || "ETHEREUM";
const provider =
  process.env.HOPR_ETHEREUM_PROVIDER ||
  "wss://kovan.infura.io/ws/v3/f7240372c1b442a6885ce9bb825ebc36";
const bootstrapAddresses =
  process.env.HOPR_BOOTSTRAP_SERVERS ||
  "/ip4/34.65.82.167/tcp/9091/p2p/16Uiu2HAm6VH37RG1R4P8hGV1Px7MneMtNc6PNPewNxCsj1HsDLXW,/ip4/34.65.111.179/tcp/9091/p2p/16Uiu2HAmPyq9Gw93VWdS3pgmyAWg2UNnrgZoYKPDUMbKDsWhzuvb";
const host = process.env.HOPR_HOST_IPV4 || "0.0.0.0:9091"; // Default IPv4

async function main() {
  const hosts: HoprOptions["hosts"] = {};
  const bootstrapAddresses = process.env.BOOTSTRAP_SERVERS.split(",");
  let addr: Multiaddr;
  let bootstrapServerMap = new Map<string, PeerInfo>();

  if (bootstrapAddresses.length == 0) {
    throw new Error(
      "Invalid bootstrap servers. Cannot start HOPR without a bootstrap node"
    );
  }

  for (let i = 0; i < bootstrapAddresses.length; i++) {
    addr = Multiaddr(bootstrapAddresses[i].trim());

    let peerInfo = bootstrapServerMap.get(addr.getPeerId());
    if (peerInfo == null) {
      peerInfo = await PeerInfo.create(
        PeerId.createFromB58String(addr.getPeerId())
      );
    }

    peerInfo.multiaddrs.add(addr);
    bootstrapServerMap.set(addr.getPeerId(), peerInfo);
  }

  let options: HoprOptions = {
    debug: Boolean(process.env.DEBUG),
    network,
    bootstrapServers: [...bootstrapServerMap.values()],
    provider,
    hosts,
  };
  NODE = await Hopr.create(options);

  NODE.on("peer:connect", (peer: PeerInfo) => {
    log(`Incoming connection from ${peer.id.toB58String()}.`);
  });

  process.once("exit", async () => {
    await NODE.down();
    return;
  });
}

main();
