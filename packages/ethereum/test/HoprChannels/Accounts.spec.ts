import { deployments, ethers } from 'hardhat'
import { singletons } from '@openzeppelin/test-helpers'
import { expect } from 'chai'
import { vmErrorMessage } from '../utils'
import { ACCOUNT_A, ACCOUNT_B, SECRET_2, SECRET_1 } from './constants'
import { AccountsMock__factory } from '../../types'

const useFixtures = deployments.createFixture(async () => {
  const [deployer] = await ethers.getSigners()

  // deploy ERC1820Registry required by ERC777 token
  await singletons.ERC1820Registry(deployer.address)

  const accounts = await new AccountsMock__factory(deployer).deploy(ethers.constants.AddressZero, '0')

  return {
    accounts
  }
})

describe('Accounts', function () {
  it('should initialize account', async function () {
    const { accounts } = await useFixtures()

    expect(accounts.initializeAccountInternal(ACCOUNT_A.address, ACCOUNT_A.uncompressedPublicKey, SECRET_2))
      .to.emit(accounts, 'AccountInitialized')
      .withArgs(ACCOUNT_A.address, ACCOUNT_A.uncompressedPublicKey, SECRET_2)

    const account = await accounts.accounts(ACCOUNT_A.address)
    expect(account.secret).to.equal(SECRET_2)
    expect(account.counter.toString()).to.equal('1')
  })

  it('should fail to initialize account when public key is wrong', async function () {
    const { accounts } = await useFixtures()

    // give wrong public key
    expect(
      accounts.initializeAccountInternal(ACCOUNT_A.address, ACCOUNT_B.uncompressedPublicKey, SECRET_1)
    ).to.be.revertedWith(vmErrorMessage('public key does not match account'))
  })

  it("should update account's secret", async function () {
    const { accounts } = await useFixtures()

    await accounts.initializeAccountInternal(ACCOUNT_A.address, ACCOUNT_A.uncompressedPublicKey, SECRET_2)

    expect(accounts.updateAccountSecretInternal(ACCOUNT_A.address, SECRET_1))
      .to.emit(accounts, 'AccountSecretUpdated')
      .withArgs(ACCOUNT_A.address, SECRET_1)

    const account = await accounts.accounts(ACCOUNT_A.address)
    expect(account.secret).to.equal(SECRET_1)
    expect(account.counter.toString()).to.equal('2')
  })

  it("should fail to update account's secret when secret is empty", async function () {
    const { accounts } = await useFixtures()

    await accounts.initializeAccountInternal(ACCOUNT_A.address, ACCOUNT_A.uncompressedPublicKey, SECRET_1)

    // give empty SECRET
    expect(accounts.updateAccountSecretInternal(ACCOUNT_A.address, ethers.constants.HashZero)).to.be.revertedWith(
      vmErrorMessage('secret must not be empty')
    )
  })

  it("should fail to update account's secret when secret is the same as before", async function () {
    const { accounts } = await useFixtures()

    await accounts.initializeAccountInternal(ACCOUNT_A.address, ACCOUNT_A.uncompressedPublicKey, SECRET_1)

    // give same SECRET
    expect(accounts.updateAccountSecretInternal(ACCOUNT_A.address, SECRET_1)).to.be.revertedWith(
      vmErrorMessage('secret must not be the same as before')
    )
  })
})
