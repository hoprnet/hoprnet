
import http, { RefinedResponse } from 'k6/http';

import { Addresses, NodeBalance } from './hoprd.types';
import { HoprNode } from './hoprd.types';
import { HoprEntityApi } from './hopr-entity-api';

/**
 * Wrap actions on hopr account
 */
export class AccountApi extends HoprEntityApi{


  public constructor(node: HoprNode) {
    super(node)
    this.addRequestHeader('entity','account');
  }

  /**
   * Invoke API Rest call for getting node addresses
   * @returns HTTP response in text mode
   */
  public getAddresses(): RefinedResponse<"text">{
    this.addRequestHeader('action','addresses');

    const response: RefinedResponse<"text">  = http.get(`${this.node.url}/account/addresses`, this.params);
    
    if(response.status === 200) {
      const addresses: Addresses = JSON.parse(response.body);
      console.log(`Hopr address for node '${this.node.alias} is' ${addresses.hopr}`)
      console.log(`Native address for node '${this.node.alias} is' ${addresses.native}`)
    }


    this.sleep(this.node.sleepTime.defaultMin, this.node.sleepTime.defaultMax);
    return response;
  }

  /**
   * Invoke API Rest call for getting node balance
   * @returns HTTP response in text mode
   */
  public getBalance(): RefinedResponse<"text">{
    this.addRequestHeader('action','balances');
  
    const response: RefinedResponse<"text">  = http.get(`${this.node.url}/account/balances`, this.params);
    
    if(response.status === 200) {      
      const balance: NodeBalance = new NodeBalance(JSON.parse(response.body));
      console.log(`Hopr balance for node '${this.node.alias} is' ${balance.hopr}`)
      console.log(`Native balance for node '${this.node.alias} is' ${balance.native}`)
    }
  
    this.sleep(this.node.sleepTime.defaultMin, this.node.sleepTime.defaultMax);
    return response;
  }

}

