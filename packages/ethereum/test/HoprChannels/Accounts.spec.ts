import { deployments } from 'hardhat'
import { expectEvent, expectRevert, constants } from '@openzeppelin/test-helpers'
import { vmErrorMessage } from '../utils'
import { formatAccount } from './utils'
import { ACCOUNT_A, ACCOUNT_B, SECRET_2, SECRET_1 } from './constants'

const Accounts = artifacts.require('AccountsMock')

const useFixtures = deployments.createFixture(async () => {
  const accounts = await Accounts.new()

  return {
    accounts
  }
})

describe('Accounts', function () {
  it('should initialize account', async function () {
    const { accounts } = await useFixtures()

    const response = await accounts.initializeAccount(
      ACCOUNT_A.address,
      ACCOUNT_A.pubKeyFirstHalf,
      ACCOUNT_A.pubKeySecondHalf,
      SECRET_2
    )

    expectEvent(response, 'AccountInitialized', {
      account: ACCOUNT_A.address,
      pubKeyFirstHalf: ACCOUNT_A.pubKeyFirstHalf,
      pubKeySecondHalf: ACCOUNT_A.pubKeySecondHalf
    })

    expectEvent(response, 'AccountSecretUpdated', {
      account: ACCOUNT_A.address,
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
      accounts.initializeAccount(ACCOUNT_A.address, ACCOUNT_B.pubKeyFirstHalf, ACCOUNT_B.pubKeySecondHalf, SECRET_1),
      vmErrorMessage('public key does not match account')
    )
  })

  it("should update account's secret", async function () {
    const { accounts } = await useFixtures()

    await accounts.initializeAccount(ACCOUNT_A.address, ACCOUNT_A.pubKeyFirstHalf, ACCOUNT_A.pubKeySecondHalf, SECRET_2)

    const response = await accounts.updateAccount(ACCOUNT_A.address, SECRET_1)

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

    await accounts.initializeAccount(ACCOUNT_A.address, ACCOUNT_A.pubKeyFirstHalf, ACCOUNT_A.pubKeySecondHalf, SECRET_1)

    // give empty SECRET
    await expectRevert(
      accounts.updateAccount(ACCOUNT_A.address, constants.ZERO_BYTES32),
      vmErrorMessage('secret must not be empty')
    )
  })

  it("should fail to update account's secret when secret is the same as before", async function () {
    const { accounts } = await useFixtures()

    await accounts.initializeAccount(ACCOUNT_A.address, ACCOUNT_A.pubKeyFirstHalf, ACCOUNT_A.pubKeySecondHalf, SECRET_1)

    // give same SECRET
    await expectRevert(
      accounts.updateAccount(ACCOUNT_A.address, SECRET_1),
      vmErrorMessage('secret must not be the same as before')
    )
  })
})
