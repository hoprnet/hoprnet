import chalk from "chalk";
import type HoprCoreConnector from "@hoprnet/hopr-core-connector-interface";
import type { Channel as ChannelInstance } from "@hoprnet/hopr-core-connector-interface";
import type Hopr from "@hoprnet/hopr-core";
import { u8aToHex } from "@hoprnet/hopr-utils";
import { AbstractCommand } from "./abstractCommand";
import type { AutoCompleteResult } from "./abstractCommand";
import { getSignedTickets } from "../utils";

export default class RedeemTicket extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super();
  }

  name() {
    return "redeemTicket";
  }

  help() {
    return "redeem a ticket";
  }

  private async checkArgs(query: string): Promise<string> {
    const [err, challange] = this._assertUsage(query, ["challange"]);
    if (err) throw new Error(err);

    return challange;
  }

  /**
   * @param query a ticket challange
   */
  async execute(query?: string): Promise<void> {
    const challange = await this.checkArgs(query ?? "");
    const { paymentChannels } = this.node;

    const ackTickets = await paymentChannels.tickets.getAll();
    const signedTickets = await getSignedTickets(
      Array.from(ackTickets.values())
    );

    const signedTicket = signedTickets.find((signedTicket) => {
      return u8aToHex(signedTicket.ticket.challenge) === challange;
    });

    if (!signedTicket) {
      console.log(chalk.yellow(`\nTicket not found.`));
      return;
    }
  }

  async autocomplete(query: string, line: string): Promise<AutoCompleteResult> {
    let channelIds: string[] = [];

    try {
      channelIds = await this.node.paymentChannels.channel.getAll(
        async (channel: ChannelInstance) => u8aToHex(await channel.channelId),
        async (channelIdsPromise: Promise<string>[]) =>
          await Promise.all(channelIdsPromise)
      );
    } catch (err) {
      console.log(chalk.red(err.message));
      return [[""], line];
    }

    if (channelIds.length < 1) {
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
