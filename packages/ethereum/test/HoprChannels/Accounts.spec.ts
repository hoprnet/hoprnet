import { expectEvent, expectRevert, constants } from '@openzeppelin/test-helpers'
import { vmErrorMessage } from '../utils'
import { formatAccount } from './utils'
import { ACCOUNT_A_ADDRESS, ACCOUNT_A_PUBKEY, ACCOUNT_B_PUBKEY, SECRET, SECRET_PRE_IMAGE } from './constants'

const Accounts = artifacts.require('AccountsMock')

describe('Accounts', function () {
  it('should initialize account', async function () {
    const accounts = await Accounts.new()

    const response = await accounts.initializeAccount(
      ACCOUNT_A_ADDRESS,
      ACCOUNT_A_PUBKEY.firstHalf,
      ACCOUNT_A_PUBKEY.secondHalf,
      SECRET
    )

    expectEvent(response, 'AccountInitialized', {
      account: ACCOUNT_A_ADDRESS,
      pubKeyFirstHalf: ACCOUNT_A_PUBKEY.firstHalf,
      pubKeySecondHalf: ACCOUNT_A_PUBKEY.secondHalf
    })

    expectEvent(response, 'AccountSecretUpdated', {
      account: ACCOUNT_A_ADDRESS,
      secret: SECRET
    })

    const account = await accounts.accounts(ACCOUNT_A_ADDRESS).then(formatAccount)
    expect(account.secret).to.equal(SECRET)
    expect(account.counter.toString()).to.equal('1')
  })

  it('should fail to initialize account when public key is wrong', async function () {
    const accounts = await Accounts.new()

    // give wrong public key
    await expectRevert(
      accounts.initializeAccount(ACCOUNT_A_ADDRESS, ACCOUNT_B_PUBKEY.firstHalf, ACCOUNT_B_PUBKEY.secondHalf, SECRET),
      vmErrorMessage('public key does not match account')
    )
  })

  it("should update account's secret", async function () {
    const accounts = await Accounts.new()

    await accounts.initializeAccount(ACCOUNT_A_ADDRESS, ACCOUNT_A_PUBKEY.firstHalf, ACCOUNT_A_PUBKEY.secondHalf, SECRET)

    const response = await accounts.updateAccount(ACCOUNT_A_ADDRESS, SECRET_PRE_IMAGE)

    expectEvent(response, 'AccountSecretUpdated', {
      account: ACCOUNT_A_ADDRESS,
      secret: SECRET_PRE_IMAGE
    })

    const account = await accounts.accounts(ACCOUNT_A_ADDRESS).then(formatAccount)
    expect(account.secret).to.equal(SECRET_PRE_IMAGE)
    expect(account.counter.toString()).to.equal('2')
  })

  it("should fail to update account's secret when secret is empty", async function () {
    const accounts = await Accounts.new()

    await accounts.initializeAccount(ACCOUNT_A_ADDRESS, ACCOUNT_A_PUBKEY.firstHalf, ACCOUNT_A_PUBKEY.secondHalf, SECRET)

    // give empty SECRET
    await expectRevert(
      accounts.updateAccount(ACCOUNT_A_ADDRESS, constants.ZERO_BYTES32),
      vmErrorMessage('secret must not be empty')
    )
  })

  it("should fail to update account's secret when secret is the same as before", async function () {
    const accounts = await Accounts.new()

    await accounts.initializeAccount(ACCOUNT_A_ADDRESS, ACCOUNT_A_PUBKEY.firstHalf, ACCOUNT_A_PUBKEY.secondHalf, SECRET)

    // give same SECRET
    await expectRevert(
      accounts.updateAccount(ACCOUNT_A_ADDRESS, SECRET),
      vmErrorMessage('secret must not be the same as before')
    )
  })
})
