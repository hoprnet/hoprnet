import Hopr from "@hoprnet/hopr-core";
import type { HoprOptions } from "@hoprnet/hopr-core";
import type HoprCoreConnector from '@hoprnet/hopr-core-connector-interface'

let NODE: Hopr<HoprCoreConnector>;

async function main() {
  const hosts: HoprOptions['hosts'] = {}
  const network = process.env.HOPR_provider || 'ETHEREUM'

  let options: HoprOptions = {
    debug: Boolean(process.env.DEBUG),
    network,
    bootstrapServers: [],
    provider: network + '_PROVIDER',
    hosts
  };
  NODE = await Hopr.create(options);
}

main();
