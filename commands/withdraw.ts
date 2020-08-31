import type HoprCoreConnector from "@hoprnet/hopr-core-connector-interface";
import type { Currencies } from "@hoprnet/hopr-core-connector-interface";
import type Hopr from "@hoprnet/hopr-core";
import {
  startDelayedInterval,
  moveDecimalPoint,
  convertPubKeyFromB58String,
  u8aToHex,
} from "@hoprnet/hopr-utils";
import chalk from "chalk";
import { AbstractCommand } from "./abstractCommand";
import { checkPeerIdInput } from "../utils";

export default class Withdraw extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super();
  }

  name() {
    return "withdraw";
  }

  help(): string {
    return "withdraw native or hopr to a specified recipient";
  }

  private async checkArgs(
    query: string
  ): Promise<{
    recipient: string;
    currency: Currencies;
    amount: string;
  }> {
    const [err, recipient, currencyRaw, amount] = this._assertUsage(query, [
      "recipient",
      "currency",
      "amount",
    ]);

    if (err) {
      throw new Error(err);
    }

    await checkPeerIdInput(recipient);

    const currency = currencyRaw.toLowerCase();

    if (!["NATIVE", "HOPR"].includes(currency.toUpperCase())) {
      throw new Error(
        `Incorrect currency provided: '${currency}', correct options are: 'native', 'hopr'.`
      );
    } else if (isNaN(Number(amount))) {
      throw new Error(`Incorrect amount provided: '${amount}'.`);
    }

    return {
      recipient,
      currency: currency.toUpperCase() as Currencies,
      amount,
    };
  }

  /**
   * Withdraws native or hopr balance.
   * @notice triggered by the CLI
   */
  async execute(query?: string): Promise<void> {
    const dispose = startDelayedInterval("Withdrawing");

    try {
      const { recipient, currency, amount } = await this.checkArgs(query ?? "");
      const { paymentChannels } = this.node;
      const pubKey = await convertPubKeyFromB58String(recipient);
      const address = await paymentChannels.utils.pubKeyToAccountId(
        pubKey.marshal()
      );

      await paymentChannels.withdraw(currency, u8aToHex(address), amount);
    } catch (err) {
      console.log(chalk.red(err.message));
    }

    dispose();
  }
}
