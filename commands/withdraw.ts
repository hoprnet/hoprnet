import type HoprCoreConnector from "@hoprnet/hopr-core-connector-interface";
import type { Currencies } from "@hoprnet/hopr-core-connector-interface";
import type Hopr from "@hoprnet/hopr-core";
import chalk from "chalk";
import { moveDecimalPoint } from "@hoprnet/hopr-utils";
import { AbstractCommand, AutoCompleteResult } from "./abstractCommand";

const _arguments = [
  "recipient (blockchain address)",
  "currency (native, hopr)",
  "amount (ETH, HOPR)",
];

export default class Withdraw extends AbstractCommand {
  constructor(public node: Hopr<HoprCoreConnector>) {
    super();
  }

  name(): string {
    return "withdraw";
  }

  help(): string {
    return "withdraw native or hopr to a specified recipient";
  }

  async autocomplete(query?: string): Promise<AutoCompleteResult> {
    return [_arguments, query ?? ""];
  }

  /**
   * Will throw if any of the arguments are incorrect.
   */
  private async checkArgs(
    query: string
  ): Promise<{
    recipient: string;
    currency: Currencies;
    amount: string;
    weiAmount: string;
  }> {
    const { NativeBalance, Balance } = this.node.paymentChannels.types;

    const [err, recipient, currencyRaw, amount] = this._assertUsage(query, [
      "recipient (blockchain address)",
      "currency (native, hopr)",
      "amount (ETH, HOPR)",
    ]);

    if (err) {
      throw new Error(err);
    }

    // @TODO: validate recipient address

    const currency = currencyRaw.toUpperCase() as Currencies;

    if (!["NATIVE", "HOPR"].includes(currency)) {
      throw new Error(
        `Incorrect currency provided: '${currency}', correct options are: 'native', 'hopr'.`
      );
    } else if (isNaN(Number(amount))) {
      throw new Error(`Incorrect amount provided: '${amount}'.`);
    }

    const weiAmount =
      currency === "NATIVE"
        ? moveDecimalPoint(amount, NativeBalance.DECIMALS)
        : moveDecimalPoint(amount, Balance.DECIMALS);

    return {
      recipient,
      currency,
      amount,
      weiAmount,
    };
  }

  /**
   * Withdraws native or hopr balance.
   * @notice triggered by the CLI
   */
  async execute(query?: string): Promise<string> {
    try {
      const { recipient, currency, amount, weiAmount } = await this.checkArgs(
        query ?? ""
      );
      const { paymentChannels } = this.node;
      const { NativeBalance, Balance } = paymentChannels.types;
      const symb = currency === "NATIVE" ? NativeBalance.SYMBOL : Balance.SYMBOL;
      await paymentChannels.withdraw(currency, recipient, weiAmount);
      return `Withdrawn ${amount} ${symb} to ${recipient}`
    } catch (err) {
      return chalk.red(err.message)
    }
  }
}
