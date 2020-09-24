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
  async execute(): Promise<string | void> {
    try {
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
        const result = await this.node.submitAcknowledgedTicket(
          ackTicket,
          index
        );

        if (result.status === "SUCCESS") {
          redeemedTickets++;
        }
      }

      return `Redeemed ${redeemedTickets} out of ${results.length} tickets.`;
    } catch (err) {
      chalk.red(err.message);
    }
  }
}
