import http, { RefinedResponse } from "k6/http"

import { SendMessageRequest } from "./hoprd.types";
import { HoprNode } from "./hoprd.types";
import { HoprEntityApi } from "./hopr-entity-api";

/**
 * Wrap actions on hopr messages
 */
export class MesssagesApi extends HoprEntityApi{

  public constructor(node: HoprNode) {
    super(node)
    this.addRequestHeader('entity','messages');
  }

  /**
   * Invoke API Rest call for sending a message
   * @returns HTTP response in text mode
   */
  public sendMessage(messageRequest: SendMessageRequest): RefinedResponse<"text">{
    this.addRequestHeader('action','messages');
    // console.log(`Sending message ${JSON.stringify(messageRequest)}`)
    const response: RefinedResponse<"text">  = http.post(`${this.node.url}/messages`, JSON.stringify(messageRequest), this.params);

    this.sleep(this.node.sleepTime.defaultMin, this.node.sleepTime.defaultMax);
    return response;
  }
}
