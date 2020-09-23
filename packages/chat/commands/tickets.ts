import BN from "bn.js";
import chalk from "chalk";
import type HoprCoreConnector from "@hoprnet/hopr-core-connector-interface";
import type { Types } from "@hoprnet/hopr-core-connector-interface";
import type Hopr from "@hoprnet/hopr-core";
import {
  u8aToHex,
  moveDecimalPoint,
  convertPubKeyFromPeerId,
} from "@hoprnet/hopr-utils";
import { GlobalState } from "./abstractCommand";
import type { AutoCompleteResult } from "./abstractCommand";
import { SendMessageBase } from "./sendMessage";
import type PeerId from "peer-id";
import {
  getMyOpenChannels,
  countSignedTickets,
  getSignedTickets,
} from "../utils";

export default class Tickets extends SendMessageBase {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super(node);
  }

  name() {
    return "tickets";
  }

  help() {
    return "lists tickets of a channel";
  }

  private async checkArgs(
    query: string,
    settings: GlobalState
  ): Promise<PeerId> {
    const [err, peerId] = this._assertUsage(query, ["PeerId"]);
    if (err) throw new Error(err);
    return await this._checkPeerId(peerId, settings);
  }

  /**
   * @param query channelId to query tickets for
   */
  public async execute(query: string, settings: GlobalState): Promise<void> {
    try {
      const { Public, Balance } = this.node.paymentChannels.types;

      const peerId = await this.checkArgs(query, settings);
      const pubKey = await convertPubKeyFromPeerId(peerId).then((res) => {
        return new Public(res.marshal());
      });

      const ackTickets = await this.node.paymentChannels.tickets.get(pubKey);
      console.log("ackTickets", ackTickets.size);

      if (ackTickets.size === 0) {
        console.log(chalk.yellow(`\nNo tickets found.`));
        return;
      }

      const { redeemed, unredeemed } = Array.from(ackTickets.values()).reduce(
        (result, ackTicket) => {
          if (ackTicket.redeemed) result.redeemed.push(ackTicket);
          else result.unredeemed.push(ackTicket);

          return result;
        },
        {
          redeemed: [],
          unredeemed: [],
        } as {
          redeemed: Types.AcknowledgedTicket[];
          unredeemed: Types.AcknowledgedTicket[];
        }
      );

      const redeemedResults = countSignedTickets(
        await getSignedTickets(redeemed)
      );
      const unredeemedResults = countSignedTickets(
        await getSignedTickets(unredeemed)
      );

      console.table(unredeemedResults.tickets);
      console.log(
        "Found",
        unredeemedResults.tickets.length,
        "unredeemed tickets for peer",
        chalk.blue(query)
      );
      console.log(
        "You will receive",
        chalk.yellow(
          moveDecimalPoint(
            unredeemedResults.total.toString(),
            Balance.DECIMALS * -1
          ).toString()
        ),
        "HOPR",
        "once you redeem them."
      );

      console.table(redeemedResults.tickets);
      console.log(
        "Found",
        redeemedResults.tickets.length,
        "redeemed tickets for peer",
        chalk.blue(query)
      );
    } catch (err) {
      console.log(err);
    }
  }

  public async autocomplete(
    query: string,
    line: string
  ): Promise<AutoCompleteResult> {
    const myAddress = await this.node.paymentChannels.account.address;
    let channelIds: string[] = [];

    try {
      const counterParties = await getMyOpenChannels(this.node);
      channelIds = await Promise.all(
        counterParties.map(async (counterParty) => {
          const pubKey = (
            await convertPubKeyFromPeerId(counterParty)
          ).marshal();
          const address = await this.node.paymentChannels.utils.pubKeyToAccountId(
            pubKey
          );
          const channelId = await this.node.paymentChannels.utils.getId(
            myAddress,
            address
          );

          return u8aToHex(channelId);
        })
      );
    } catch (err) {
      console.log(chalk.red(err.message));
      return [[""], line];
    }

    if (channelIds.length === 0) {
      console.log(chalk.red(`\nNo open channels found.`));
      return [[""], line];
    }

    const hits = query
      ? channelIds.filter((channelId: string) => channelId.startsWith(query))
      : channelIds;

    return [
      hits.length ? hits.map((str: string) => `tickets ${str}`) : ["tickets"],
      line,
    ];
  }
}
