import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";

const S3_PROGRAM_END = 1658836800

const main: DeployFunction = async function ({
    ethers, deployments, getNamedAccounts, artifacts
}: HardhatRuntimeEnvironment) {
  const { deploy } = deployments;
  const { utils } = ethers;
  const { deployer, admin } = await getNamedAccounts();

  const HoprBoost = await deployments.get("HoprBoost");
  const xHOPR = await deployments.get("xHoprMock");
  const wxHOPR = await deployments.get("HoprToken");

    // check the lastest block timestamp
    const latestBlockTimestamp = (await ethers.provider.getBlock('latest')).timestamp
    console.log(`Latest block timestamp is ${latestBlockTimestamp}`)

    let stakeContract;
    if (latestBlockTimestamp <= S3_PROGRAM_END) {
        // deploy season 3
        stakeContract = await deploy("HoprStake", {
            contract: "HoprStakeSeason3",
            from: deployer,
            args: [HoprBoost.address, admin, xHOPR.address, wxHOPR.address],
            log: true,
        });
    } else {
        // deploy season 4
        stakeContract = await deploy("HoprStake", {
            contract: "HoprStakeSeason4",
            from: deployer,
            args: [HoprBoost.address, admin, xHOPR.address, wxHOPR.address],
            log: true,
        });
    }
};

main.tags = ['HoprStake'];
main.dependencies = ['HoprBoost', 'HoprToken'];
main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production || !!env.network.tags.staging

export default main;
