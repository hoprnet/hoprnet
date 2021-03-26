import { deployments } from 'hardhat'
import { singletons, expectEvent, expectRevert, constants } from '@openzeppelin/test-helpers'
import { vmErrorMessage } from '../utils'
import { formatAccount } from './utils'
import { ACCOUNT_A, ACCOUNT_B, SECRET_2, SECRET_1 } from './constants'

const Accounts = artifacts.require('AccountsMock')

const useFixtures = deployments.createFixture(async () => {
  const [deployer] = await web3.eth.getAccounts()

  // deploy ERC1820Registry required by ERC777 token
  await singletons.ERC1820Registry(deployer)

  const accounts = await Accounts.new(constants.ZERO_ADDRESS, '0')

  return {
    accounts
  }
})

describe('Accounts', function () {
  it('should initialize account', async function () {
    const { accounts } = await useFixtures()

    const response = await accounts.initializeAccountInternal(ACCOUNT_A.address, ACCOUNT_A.uncompressedPubKey, SECRET_2)

    expectEvent(response, 'AccountInitialized', {
      account: ACCOUNT_A.address,
      uncompressedPubKey: ACCOUNT_A.uncompressedPubKey,
      secret: SECRET_2
    })

    const account = await accounts.accounts(ACCOUNT_A.address).then(formatAccount)
    expect(account.secret).to.equal(SECRET_2)
    expect(account.counter.toString()).to.equal('1')
  })

  it('should fail to initialize account when public key is wrong', async function () {
    const { accounts } = await useFixtures()

    // give wrong public key
    await expectRevert(
      accounts.initializeAccountInternal(ACCOUNT_A.address, ACCOUNT_B.uncompressedPubKey, SECRET_1),
      vmErrorMessage('public key does not match account')
    )
  })

  it("should update account's secret", async function () {
    const { accounts } = await useFixtures()

    await accounts.initializeAccountInternal(ACCOUNT_A.address, ACCOUNT_A.uncompressedPubKey, SECRET_2)

    const response = await accounts.updateAccountSecretInternal(ACCOUNT_A.address, SECRET_1)

    expectEvent(response, 'AccountSecretUpdated', {
      account: ACCOUNT_A.address,
      secret: SECRET_1
    })

    const account = await accounts.accounts(ACCOUNT_A.address).then(formatAccount)
    expect(account.secret).to.equal(SECRET_1)
    expect(account.counter.toString()).to.equal('2')
  })

  it("should fail to update account's secret when secret is empty", async function () {
    const { accounts } = await useFixtures()

    await accounts.initializeAccountInternal(ACCOUNT_A.address, ACCOUNT_A.uncompressedPubKey, SECRET_1)

    // give empty SECRET
    await expectRevert(
      accounts.updateAccountSecretInternal(ACCOUNT_A.address, constants.ZERO_BYTES32),
      vmErrorMessage('secret must not be empty')
    )
  })

  it("should fail to update account's secret when secret is the same as before", async function () {
    const { accounts } = await useFixtures()

    await accounts.initializeAccountInternal(ACCOUNT_A.address, ACCOUNT_A.uncompressedPubKey, SECRET_1)

    // give same SECRET
    await expectRevert(
      accounts.updateAccountSecretInternal(ACCOUNT_A.address, SECRET_1),
      vmErrorMessage('secret must not be the same as before')
    )
  })
})
