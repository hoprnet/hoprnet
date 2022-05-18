import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";

const main: DeployFunction = async function (hre: HardhatRuntimeEnvironment) {
  const { deployments, getNamedAccounts } = hre;

  const { deploy } = deployments;
  const { deployer } = await getNamedAccounts();

  await deploy("xHoprMock", {
    contract: "ERC677Mock",
    from: deployer,
    log: true,
  });
};
main.tags = ['xHoprMock'];
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production

export default main;
