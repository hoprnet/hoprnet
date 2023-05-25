import { TASK_ETHERSCAN_VERIFY } from "hardhat-deploy";
import { DeployFunction } from "hardhat-deploy/types";
import { HardhatRuntimeEnvironment } from "hardhat/types";

const func: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { run } = hre;
  if (!["rinkeby", "mainnet"].includes(hre.network.name)) {
    return;
  }

  if (!process.env.INFURA_KEY) {
    console.log(
      `Could not find Infura key in env, unable to connect to network ${hre.network.name}`
    );
    return;
  }

  console.log("Verification of Roles Modifier in etherscan...");
  console.log("Waiting for 1 minute before verifying contracts...");
  // Etherscan needs some time to process before trying to verify.
  await new Promise((resolve) => setTimeout(resolve, 60000));

  console.log("Starting to verify now");

  await run(TASK_ETHERSCAN_VERIFY, {
    apiKey: process.env.ETHERSCAN_KEY_API,
    license: "GPL-3.0",
    solcInput: true,
    forceLicense: true, // we need this because contracts license is LGPL-3.0-only
  });
};
export default func;
