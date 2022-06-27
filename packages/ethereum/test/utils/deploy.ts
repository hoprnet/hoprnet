import { ethers } from 'hardhat'
import { Contract, Signer } from 'ethers'

export const deployContractFromFactory = async (
  deployer: Signer,
  contractName: string,
  args?: any[]
): Promise<Contract> => {
  const contract = await ethers.getContractFactory(contractName)
  const artifact = !!args ? await contract.connect(deployer).deploy(...args) : await contract.connect(deployer).deploy()
  return artifact.deployed()
}
