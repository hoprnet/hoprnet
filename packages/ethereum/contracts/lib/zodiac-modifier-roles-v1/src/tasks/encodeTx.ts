import "hardhat-deploy";
import "@nomiclabs/hardhat-ethers";
import { task, types } from "hardhat/config";

task(
  "encodeTx",
  "encodes the transaction data to be used as the _data parameter of requireToPassMessage()."
)
  .addParam(
    "to",
    "Address that the Safe should send a Tx to",
    undefined,
    types.string
  )
  .addParam(
    "value",
    "Value of the transaction, in wei",
    undefined,
    types.string
  )
  .addParam("data", "Transaction data", undefined, types.string)
  .addParam("operation", "(0) Call | (1) DelegateCall", undefined, types.string)
  .setAction(async (taskArgs, hardhatRuntime) => {
    const [caller] = await hardhatRuntime.ethers.getSigners();
    console.log("Using the account:", caller.address);
    const module = await hardhatRuntime.ethers.getContractAt(
      "SafeBridgeModule",
      "0x0000000000000000000000000000000000000000"
    );
    const moduleTx = await module.populateTransaction.executeTransaction(
      taskArgs.to,
      `${taskArgs.value}`,
      taskArgs.data,
      `${taskArgs.operation}`
    );

    console.log("Encoded Transaction:", moduleTx.data);
  });

task(
  "encodeBridgeTx",
  "encodes the data for a requireToPassMessage() call to the AMB contract."
)
  .addParam("to", "Address of the AMB module", undefined, types.string)
  .addParam("data", "Transaction data", undefined, types.string)
  .addParam(
    "gas",
    "Amount of gas to use for the transaction on the other side of the AMB",
    undefined,
    types.string
  )
  .setAction(async (taskArgs, hardhatRuntime) => {
    const [caller] = await hardhatRuntime.ethers.getSigners();
    console.log("Using the account:", caller.address);
    const amb = await hardhatRuntime.ethers.getContractAt(
      "IAMB",
      "0x0000000000000000000000000000000000000000"
    );
    const ambTx = await amb.populateTransaction.requireToPassMessage(
      taskArgs.to,
      taskArgs.data,
      `${taskArgs.gas}`
    );

    console.log("Encoded Transaction:", ambTx.data);
  });

export {};
