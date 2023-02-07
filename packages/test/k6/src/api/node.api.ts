import http, { RefinedResponse } from 'k6/http';

import { HoprNode, PeerResponse } from './hoprd.types';
import { HoprEntityApi } from './hopr-entity-api';

/**
 * Wrap actions on hopr nodes
 */
export class NodeApi extends HoprEntityApi{


  public constructor(node: HoprNode) {
    super(node)
    this.addRequestHeader('entity','node');
  }

  /**
   * Invoke API Rest call for getting the current status of the node
   * @returns HTTP response in text mode
   */
  public getInfo(): RefinedResponse<"text">{
    this.addRequestHeader('action','info');

    const response: RefinedResponse<"text">  = http.get(`${this.node.url}/node/info`, this.params);   
    
    this.sleep(this.node.sleepTime.defaultMin, this.node.sleepTime.defaultMax);
    return response;
  }

  /**
   * Invoke API Rest call for getting node peers
   * @returns HTTP response in text mode
   */
  public getPeers(): RefinedResponse<"text"> {
    this.addRequestHeader('action','peers');
  
    const response: RefinedResponse<"text">  = http.get(`${this.node.url}/node/peers`, this.params);
    if(response.status === 200 ) {
      const peersResponse: PeerResponse = JSON.parse(response.body);
      console.log(`Node ${this.node.alias} got ${peersResponse.announced.length} announced peers`)
    }
     
    this.sleep(this.node.sleepTime.defaultMin, this.node.sleepTime.defaultMax);
    return response;
  }

}
