import { ethers } from 'hardhat'
import { Contract, Signer, constants, BigNumber } from 'ethers'
import { expect } from 'chai'
import { deployContractFromFactory, shouldSupportInterfaces } from '../utils'
import { BADGES, baseURI, MINTER_ROLE, NAME, SYMBOL } from '../utils/constants'

describe('HoprBoost NFT', function () {
  let deployer: Signer
  let admin: Signer
  let minter2: Signer
  let goldHodlers: Signer[]
  let silverHodlers: Signer[]
  let testnetParticipants: Signer[]

  let deployerAddress: string
  let adminAddress: string
  let minter2Address: string
  let goldHodlerAddresses: string[]
  let silverHodlerAddresses: string[]
  let testnetParticipantAddresses: string[]
  let nftContract: Contract

  const reset = async () => {
    let signers: Signer[]
    ;[deployer, admin, minter2, ...signers] = await ethers.getSigners()
    goldHodlers = signers.slice(3, 5)
    silverHodlers = signers.slice(5, 7)
    testnetParticipants = signers.slice(7, 10)

    deployerAddress = await deployer.getAddress()
    adminAddress = await admin.getAddress()
    minter2Address = await minter2.getAddress()
    goldHodlerAddresses = await Promise.all(goldHodlers.map((h) => h.getAddress()))
    silverHodlerAddresses = await Promise.all(silverHodlers.map((h) => h.getAddress()))
    testnetParticipantAddresses = await Promise.all(testnetParticipants.map((h) => h.getAddress()))

    // create NFT
    nftContract = await deployContractFromFactory(deployer, 'HoprBoost', [adminAddress, ''])
    // add a minter
    await nftContract.connect(admin).grantRole(MINTER_ROLE, minter2Address)

    // -----logs
    console.table([
      ['Deployer', deployerAddress],
      ['Admin', adminAddress],
      ['Minter 2', minter2Address],
      ['Gold Hodler', JSON.stringify(goldHodlerAddresses)],
      ['Silver Hodler', JSON.stringify(silverHodlerAddresses)],
      ['Testnet Participant', JSON.stringify(testnetParticipantAddresses)]
    ])
  }
  describe('integration tests', function () {
    before(async function () {
      await reset()
    })

    it('has correct name', async function () {
      expect((await nftContract.name()).toString()).to.equal(NAME)
    })

    it('has correct symbol', async function () {
      expect((await nftContract.symbol()).toString()).to.equal(SYMBOL)
    })

    it('has total supply of zero', async function () {
      expect((await nftContract.totalSupply()).toString()).to.equal(constants.Zero.toString())
    })

    it('has no boost factor', async function () {
      expect((await nftContract.boostOf(constants.Zero)).toString()).to.equal([constants.Zero, constants.Zero].join())
    })

    it('has type zero for all tokens', async function () {
      expect((await nftContract.typeIndexOf(constants.Zero)).toString()).to.equal(constants.Zero.toString())
      expect((await nftContract.typeIndexOf(constants.One)).toString()).to.equal(constants.Zero.toString())
      expect((await nftContract.typeIndexOf(constants.Two)).toString()).to.equal(constants.Zero.toString())
    })

    describe('mint', function () {
      it('allows admin - a minter - to mint', async function () {
        await expect(
          nftContract
            .connect(admin)
            .mint(goldHodlerAddresses[0], BADGES[0].type, BADGES[0].rank, BADGES[0].nominator, BADGES[0].deadline)
        )
          .to.emit(nftContract, 'BoostMinted')
          .withArgs(constants.One.toString(), BADGES[0].nominator.toString(), BADGES[0].deadline.toString())
          .to.emit(nftContract, 'Transfer')
          .withArgs(constants.AddressZero, goldHodlerAddresses[0], constants.Zero.toString())
      })

      it('has total supply of one', async function () {
        expect((await nftContract.totalSupply()).toString()).to.equal(constants.One.toString())
      })

      it('has correct boost factor', async function () {
        expect((await nftContract.boostOf(constants.Zero)).toString()).to.equal(
          [BADGES[0].nominator, BADGES[0].deadline].join()
        )
      })

      it('has correct type', async function () {
        expect((await nftContract.typeIndexOf(constants.Zero)).toString()).to.equal(constants.One.toString())
      })

      it('can find type by token Id', async function () {
        expect((await nftContract.typeOf(constants.Zero)).toString()).to.equal(BADGES[0].type)
      })

      it('can find type by type index', async function () {
        expect((await nftContract.typeAt(constants.One)).toString()).to.equal(BADGES[0].type)
      })
    })

    describe('baseURI', function () {
      it('fails when non admin sets URI', async function () {
        await expect(nftContract.connect(deployer).updateBaseURI(baseURI)).to.be.revertedWith(
          `AccessControl: account ${deployerAddress.toLowerCase()} is missing role ${constants.HashZero}`
        )
      })

      it('minted tokens has the corect URI', async function () {
        expect((await nftContract.tokenURI(constants.Zero)).toString()).to.equal(
          `${''}${BADGES[0].type}/${BADGES[0].rank}`
        )
      })

      it('admin can set', async function () {
        await nftContract.connect(admin).updateBaseURI(baseURI)
      })

      it('minted tokens has the updated URI', async function () {
        expect((await nftContract.tokenURI(constants.Zero)).toString()).to.equal(
          `${baseURI}${BADGES[0].type}/${BADGES[0].rank}`
        )
      })
    })

    describe('mint one token of existing type', function () {
      it('allows a minter - to mint', async function () {
        await nftContract
          .connect(minter2)
          .mint(goldHodlerAddresses[1], BADGES[0].type, BADGES[0].rank, BADGES[0].nominator, BADGES[0].deadline)
      })

      it('has total supply of two', async function () {
        expect((await nftContract.totalSupply()).toString()).to.equal(constants.Two.toString())
      })

      it('has correct boost factor', async function () {
        expect((await nftContract.boostOf(constants.One)).toString()).to.equal(
          [BADGES[0].nominator, BADGES[0].deadline].join()
        )
      })

      it('has correct type', async function () {
        expect((await nftContract.typeIndexOf(constants.One)).toString()).to.equal(constants.One.toString())
      })
    })

    describe('batch mint an existing type', function () {
      it('allows a minter - to mint', async function () {
        await nftContract
          .connect(minter2)
          .batchMint(silverHodlerAddresses, BADGES[1].type, BADGES[1].rank, BADGES[1].nominator, BADGES[1].deadline)
      })

      it('has total supply of four', async function () {
        expect((await nftContract.totalSupply()).toString()).to.equal(BigNumber.from('4').toString())
      })

      it('has correct boost factor', async function () {
        expect((await nftContract.boostOf(constants.Two)).toString()).to.equal(
          [BADGES[1].nominator, BADGES[1].deadline].join()
        )
      })

      it('has correct type', async function () {
        expect((await nftContract.typeIndexOf(BigNumber.from('3'))).toString()).to.equal(constants.One.toString())
      })
    })

    describe('batch mint a new type', function () {
      it('allows a minter - to mint', async function () {
        await nftContract
          .connect(minter2)
          .batchMint(
            testnetParticipantAddresses,
            BADGES[2].type,
            BADGES[2].rank,
            BADGES[2].nominator,
            BADGES[2].deadline
          )
      })

      it('has total supply of seven', async function () {
        expect((await nftContract.totalSupply()).toString()).to.equal(BigNumber.from('7').toString())
      })

      it('has correct boost factor', async function () {
        expect((await nftContract.boostOf(BigNumber.from('6'))).toString()).to.equal(
          [BADGES[2].nominator, BADGES[2].deadline].join()
        )
      })

      it('has correct type', async function () {
        expect((await nftContract.typeIndexOf(BigNumber.from('6'))).toString()).to.equal(constants.Two.toString())
      })
    })

    describe('should support interface', function () {
      it('has correct type', async function () {
        const contract = await deployContractFromFactory(deployer, 'HoprBoost', [adminAddress, baseURI])
        shouldSupportInterfaces(contract, [
          'IHoprBoost',
          'ERC165',
          'AccessControlEnumerable',
          'ERC721',
          'ERC721Enumerable'
        ])
      })
    })

    describe('claim ERC20 tokens from the NFT contract', function () {
      let erc20
      before(async function () {
        erc20 = await deployContractFromFactory(deployer, 'ERC20Mock', [nftContract.address, constants.One])
      })

      it('has one mock erc20 token', async function () {
        expect((await erc20.balanceOf(nftContract.address)).toString()).to.equal(constants.One.toString())
      })

      it('admin can reclaim erc20 token', async function () {
        await nftContract.connect(admin).reclaimErc20Tokens(erc20.address)
        expect((await erc20.balanceOf(nftContract.address)).toString()).to.equal(constants.Zero.toString())
        expect((await erc20.balanceOf(adminAddress)).toString()).to.equal(constants.One.toString())
      })
    })

    describe('claim ERC721 tokens from the NFT contract', function () {
      let erc721
      before(async function () {
        const contract = await ethers.getContractFactory('ERC721Mock')
        const artifact = await contract.connect(deployer).deploy()
        erc721 = await artifact.deployed()
        await erc721.mint(nftContract.address, 3)
      })

      it('has one mock erc721 token', async function () {
        expect((await erc721.balanceOf(nftContract.address)).toString()).to.equal(constants.One.toString())
      })

      it('admin can reclaim erc721 token', async function () {
        await nftContract.connect(admin).reclaimErc721Tokens(erc721.address, 3)
        expect((await erc721.balanceOf(nftContract.address)).toString()).to.equal(constants.Zero.toString())
        expect((await erc721.balanceOf(adminAddress)).toString()).to.equal(constants.One.toString())
      })
    })
  })
})
