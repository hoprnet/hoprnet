import chalk from "chalk";
import type HoprCoreConnector from "@hoprnet/hopr-core-connector-interface";
import type Hopr from "@hoprnet/hopr-core";
import { AbstractCommand } from "./abstractCommand";

export default class RedeemTickets extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super();
  }

  name() {
    return "redeemTickets";
  }

  help() {
    return "redeem tickets";
  }

  /**
   * @param query a ticket challange
   */
  async execute(query?: string): Promise<string | void> {
    try {
      const amount = Number(query ?? 1);
      if (isNaN(amount)) throw Error(`Amount passed is invalid ${amount}.`);

      // get only unredeemed tickets
      const results = await this.node
        .getAcknowledgedTickets()
        .then((tickets) => {
          return tickets.filter((ticket) => !ticket.ackTicket.redeemed);
        });

      if (results.length === 0) {
        return "No unredeemed tickets found.";
      }

      let redeemedTickets = 0;
      for (const { ackTicket, index } of results) {
        try {
          // @TODO: handle when a ticketRedeemption fails due to network, etc
          ackTicket.redeemed = true;
          await this.node.updateAcknowledgedTicket(ackTicket, index);
          await this.node.paymentChannels.channel.tickets.submit(ackTicket);
          redeemedTickets++;
        } catch {
          // @TODO: handle this error
        }
      }

      return `Redeemed ${redeemedTickets} out of ${results.length} tickets.`;
    } catch (err) {
      chalk.red(err.message);
    }
  }
}
