import "hardhat-deploy";
import "@nomiclabs/hardhat-ethers";
// import { deployAndSetUpModule } from "@gnosis.pm/zodiac";
import { task, types } from "hardhat/config";
// import { HardhatRuntimeEnvironment } from "hardhat/types";

// interface RolesTaskArgs {
//   owner: string;
//   avatar: string;
//   target: string;
//   proxied: boolean;
// }

// const deployRolesModifier = async (
//   taskArgs: RolesTaskArgs,
//   hardhatRuntime: HardhatRuntimeEnvironment
// ) => {
//   const [caller] = await hardhatRuntime.ethers.getSigners();
//   console.log("Using the account:", caller.address);
//   const Modifier = await hardhatRuntime.ethers.getContractFactory("Roles");

//   if (taskArgs.proxied) {
//     const chainId = await hardhatRuntime.getChainId();
//     const { transaction } = deployAndSetUpModule(
//       "roles",
//       {
//         values: [taskArgs.owner, taskArgs.avatar, taskArgs.target],
//         types: ["address", "address", "address"],
//       },
//       hardhatRuntime.ethers.provider,
//       Number(chainId),
//       Date.now().toString()
//     );
//     const deploymentTransaction = await caller.sendTransaction(transaction);
//     const receipt = await deploymentTransaction.wait();
//     console.log("Modifier contract deployed to", receipt.logs[1].address);
//     return;
//   }

//   const modifier = await Modifier.deploy(
//     taskArgs.owner,
//     taskArgs.avatar,
//     taskArgs.target
//   );

//   console.log("Modifier deployed to:", modifier.address);
// };

// task("setup", "Deploys a Roles modifier")
//   .addParam("owner", "Address of the owner", undefined, types.string)
//   .addParam(
//     "avatar",
//     "Address of the avatar (e.g. Safe)",
//     undefined,
//     types.string
//   )
//   .addParam("target", "Address of the target", undefined, types.string)
//   .addParam(
//     "proxied",
//     "Deploys contract through factory",
//     false,
//     types.boolean,
//     true
//   )
//   .setAction(deployRolesModifier);

task("verifyEtherscan", "Verifies the contract on etherscan")
  .addParam("modifier", "Address of the modifier", undefined, types.string)
  .addParam("owner", "Address of the owner", undefined, types.string)
  .addParam(
    "avatar",
    "Address of the avatar (e.g. Safe)",
    undefined,
    types.string
  )
  .setAction(async (taskArgs, hardhatRuntime) => {
    await hardhatRuntime.run("verify", {
      address: taskArgs.modifier,
      constructorArgsParams: [taskArgs.owner, taskArgs.avatar, taskArgs.target],
    });
  });

export {};
