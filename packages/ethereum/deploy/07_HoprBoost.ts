import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";

const PROTOCOL_CONFIG = require('../../core/protocol-config.json')

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { ethers, deployments, getNamedAccounts, network, environment } = hre;

  const environmentConfig = PROTOCOL_CONFIG.environments[environment]

  const { deploy } = deployments;
  const { deployer, admin } = await getNamedAccounts();

  const hoprBoostContract = await deploy("HoprBoost", {
    from: deployer,
    args: [admin, ""],
    log: true,
  });

  // get hoprboost types created in the production contract
  const productionHoprBoost = await deployments.get('HoprBoost');
  console.log(`productionHoprBoost ${productionHoprBoost.address} and just deployed ${hoprBoostContract.address}`)
};
main.tags = ['HoprBoost'];
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production || !!env.network.tags.staging

export default main;
