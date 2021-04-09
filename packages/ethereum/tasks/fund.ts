import type { HardhatRuntimeEnvironment, RunSuperFunction } from 'hardhat/types'
//import { HoprToken__factory } from '../types'


const send = (signer, txparams) =>
  signer.sendTransaction(txparams, (error, transactionHash) => {
    if (error) {
      console.log(`Error: ${error}`);
    }
    console.log(`transactionHash: ${transactionHash}`);
  });

/**
 * Funds an account with HOPR
 */
async function main(
  { address, amount }: { address: string; amount: string },
  { ethers, network }: HardhatRuntimeEnvironment,
  _runSuper: RunSuperFunction<any>
) {
  console.log('💰 Starting fund task', {
    address,
    network: network.name
  })
  const etherAmount = '1'
  const signer = ethers.provider.getSigner()
  const tx = {
    to: address,
    value: ethers.utils.parseEther(etherAmount)
  };
  // const hoprToken = HoprToken__factory.connect(address, ethers.provider).connect(signer)

  console.log(`💵 Sending ${etherAmount} ETH to ${address} on network ${network.name}`);
  await send(signer, tx);

  console.log(`[TODO] 💵 Sending ${amount} HOPR to ${address} on network ${network.name}`);
  // await hoprToken.mint(await signer.getAddress(), amount, ethers.constants.HashZero, ethers.constants.HashZero, {
  //   from: signer.getAddress(),
  //   gasLimit: 200e3
  // })
}

export default main
