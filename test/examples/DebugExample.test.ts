import {
  HoprTokenContract,
  HoprTokenInstance
} from "../../types/truffle-contracts";
import { Debug } from "../../types/truffle";

const HoprToken: HoprTokenContract = artifacts.require("HoprToken");
const debug: Debug = global["debug"];

contract.skip("DebugExample.test", _accounts => {
  let hoprToken: HoprTokenInstance;

  before(async () => {
    hoprToken = await HoprToken.deployed();
  });

  it("should launch debugger", async () => {
    await debug(hoprToken.name());
    assert.isTrue(true);
  });
});
