import * as hre from 'hardhat'
import { BigNumber, constants, Contract, Signer, utils } from 'ethers'
import{ expect } from "chai";
import deployERC1820Registry from '../../deploy/01_ERC1820Registry'
import { advanceTimeForNextBlock, calculateRewards, deployContractFromFactory, latestBlockTime } from '../utils'

describe('HoprStake2', function () {
    let deployer: Signer;
    let admin: Signer;
    let participants: Signer[];

    let deployerAddress: string;
    let adminAddress: string;
    let participantAddresses: string[];

    let nftContract: Contract;
    let stakeContract: Contract;
    let erc677: Contract;
    let erc777: Contract;

    const BASE_URI = 'https://stake.hoprnet.org/'
    const PROGRAM_S2_START = 1642424400; // Jan 17 2022 14:00 CET.
    const PROGRAM_S2_END = 1650974400; // Apr 26th 2022 14:00 CET.
    const BASIC_FACTOR_NUMERATOR = 5787
    const BADGES = [
        {
            type: "HODLr",
            rank: "silver",
            deadline: PROGRAM_S2_START,
            nominator: "158" // 0.5% APY
        },
        {
            type: "HODLr",
            rank: "platinum",
            deadline: PROGRAM_S2_END,
            nominator: "317" // 1% APY
        },
        {
            type: "Past",
            rank: "gold",
            deadline: 123456, // sometime long long ago
            nominator: "100"
        },
        {
            type: "HODLr",
            rank: "bronze extra",
            deadline: PROGRAM_S2_END,
            nominator: "79" // 0.25% APY
        },
        {
            type: "Testnet participant",
            rank: "gold",
            deadline: PROGRAM_S2_END,
            nominator: "317" // 0.25% APY
        },
    ];

    const reset = async () => {
        let signers: Signer[];
        [deployer, admin, ...signers] = await hre.ethers.getSigners();
        participants = signers.slice(3,6); // 3 participants

        deployerAddress = await deployer.getAddress();
        adminAddress = await admin.getAddress();
        participantAddresses = await Promise.all(participants.map(h => h.getAddress()));

        // set 1820 registry
        await deployERC1820Registry(hre, deployer)
        // set stake and reward tokens
        erc677 = await deployContractFromFactory(deployer, "ERC677Mock");
        // erc777 is the reward token (wxHOPR). admin account holds 5 million reward tokens
        erc777 = await deployContractFromFactory(deployer, 'ERC777Mock', [
            adminAddress,
            utils.parseUnits('5000000', 'ether'),
            'ERC777Mock',
            'M777',
            [adminAddress]
        ])

        // create NFT and stake contract
        nftContract = await deployContractFromFactory(deployer, "HoprBoost", [adminAddress, BASE_URI]);
        stakeContract = await deployContractFromFactory(deployer, "HoprStake2", [nftContract.address, adminAddress, erc677.address, erc777.address]);

        // airdrop some NFTs (0,1,2,3) to participants
        await nftContract.connect(admin).batchMint(participantAddresses.slice(0, 2), BADGES[0].type, BADGES[0].rank, BADGES[0].nominator, BADGES[0].deadline);
        await nftContract.connect(admin).mint(participantAddresses[0], BADGES[1].type, BADGES[1].rank, BADGES[1].nominator, BADGES[1].deadline);
        await nftContract.connect(admin).mint(participantAddresses[0], BADGES[4].type, BADGES[4].rank, BADGES[4].nominator, BADGES[4].deadline);

        // airdrop some ERC677 to participants
        await erc677.batchMintInternal(participantAddresses, utils.parseUnits('10000', 'ether')); // each participant holds 10k xHOPR
        // stake some tokens
        await erc677.connect(participants[0]).transferAndCall(stakeContract.address, utils.parseUnits('1000', 'ether'), '0x'); // stake 1000 LOCK_TOKEN
        // redeem a HODLr token - silver
        await nftContract.connect(participants[0]).functions["safeTransferFrom(address,address,uint256)"](participantAddresses[0], stakeContract.address, 0);
        // redeem a HODLr token - platinum
        await nftContract.connect(participants[0]).functions["safeTransferFrom(address,address,uint256)"](participantAddresses[0], stakeContract.address, 2);
        // redeem a Testnet participant token - gold
        await nftContract.connect(participants[0]).functions["safeTransferFrom(address,address,uint256)"](participantAddresses[0], stakeContract.address, 3);
        // provide 5 million REWARD_TOKEN
        await erc777.connect(admin).send(stakeContract.address, utils.parseUnits('5000000', 'ether'), '0x'); 
    }

    describe('Stake season 2', function () {
        beforeEach(async function () {
            await reset();
        })

        describe('For whitelisting', function () {
            describe('redeemed token', function () {
                it('can get redeemed token with isNftTypeAndRankRedeemed1', async function () {
                    const isNftTypeAndRankRedeemed1 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed1(BADGES[0].type, BADGES[0].rank, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed1).to.equal(true);
                });
                it('can get redeemed token with isNftTypeAndRankRedeemed2', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed2 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed2(1, BADGES[0].rank, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed2).to.equal(true);
                });
                it('can get redeemed token with isNftTypeAndRankRedeemed3', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed3 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed3(1, BADGES[0].nominator, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed3).to.equal(true);
                });
                it('can get redeemed token with isNftTypeAndRankRedeemed4', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed4 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed4(BADGES[0].type, BADGES[0].nominator, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed4).to.equal(true);
                });
                it('can get redeemed token with isNftTypeAndRankRedeemed4', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed4 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed4(BADGES[0].type, BADGES[0].nominator, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed4).to.equal(true);
                });
            });
            describe('redeemed token but wrong info', function () {
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed1, differnt rank', async function () {
                    const isNftTypeAndRankRedeemed1 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed1(BADGES[0].type, 'diamond', participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed1).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed1, different type', async function () {
                    const isNftTypeAndRankRedeemed1 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed1('Rando type', BADGES[0].rank, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed1).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed2, different rank', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed2 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed2(1, 'diamond', participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed2).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed2, different type', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed2 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed2(2, BADGES[0].rank, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed2).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed3, differnt factor', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed3 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed3(1, 888, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed3).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed4, different type', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed3 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed3(2, BADGES[0].nominator, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed3).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed4, different factor', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed4 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed4(BADGES[0].type, 888, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed4).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed4, different type', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed4 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed4('Rando type', BADGES[0].nominator, participantAddresses[0]);
                    expect(isNftTypeAndRankRedeemed4).to.equal(false);
                });
            });
            describe('owned but not redeemed token', function () {
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed1', async function () {
                    const isNftTypeAndRankRedeemed1 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed1(BADGES[0].type, BADGES[0].rank, participantAddresses[1]);
                    expect(isNftTypeAndRankRedeemed1).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed2', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed2 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed2(1, BADGES[0].rank, participantAddresses[1]);
                    expect(isNftTypeAndRankRedeemed2).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed3', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed3 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed3(1, BADGES[0].nominator, participantAddresses[1]);
                    expect(isNftTypeAndRankRedeemed3).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed4', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed4 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed4(BADGES[0].type, BADGES[0].nominator, participantAddresses[1]);
                    expect(isNftTypeAndRankRedeemed4).to.equal(false);
                });
                it('should be false, when getting redeemed token with isNftTypeAndRankRedeemed4', async function () {
                    // type index starts from 1
                    const isNftTypeAndRankRedeemed4 = await stakeContract.connect(deployer).isNftTypeAndRankRedeemed4(BADGES[0].type, BADGES[0].nominator, participantAddresses[1]);
                    expect(isNftTypeAndRankRedeemed4).to.equal(false);
                });
            });
        });
    });

    describe('Before season 2 starts', function () {
        before(async function () {
            await reset();
            // mint nft nr 4
            await nftContract.connect(admin).mint(participantAddresses[1], BADGES[0].type, BADGES[0].rank, BADGES[0].nominator, BADGES[0].deadline);
        })
        it('has not start season 2 to PROGRAM_S2_START', async function () {
            const blockTime = await latestBlockTime(hre.ethers.provider)
            expect(blockTime).to.be.lte(PROGRAM_S2_START)
        })
        it('can redeem HODLr token', async function () {
          await expect(
            nftContract
              .connect(participants[1])
              .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[1], stakeContract.address, 4)
          )
            .to.emit(stakeContract, 'Redeemed')
            .withArgs(participantAddresses[1], '4', true)
        })

        it('can receive LOCK_TOKEN with `transferAndCall()`', async () => {
          expect((await erc677.balanceOf(participantAddresses[0])).toString()).to.equal(
            utils.parseUnits('9000', 'ether').toString()
          )
          await erc677
            .connect(participants[0])
            .transferAndCall(stakeContract.address, utils.parseUnits('1000', 'ether'), '0x') // stake 1000 LOCK_TOKEN
          expect((await erc677.balanceOf(participantAddresses[0])).toString()).to.equal(
            utils.parseUnits('8000', 'ether').toString()
          )
        })
        it('has no cumulated rewards increment', async () => {
          const blockTime = await latestBlockTime(hre.ethers.provider)
          console.log('debug blockTime', blockTime)
          const rewardsIncrement = await stakeContract.getCumulatedRewardsIncrement(participantAddresses[0])
          expect(BigNumber.from(rewardsIncrement).toString()).to.equal(constants.Zero.toString())
        })
        it('has nothing to claim', async () => {
          await expect(stakeContract.claimRewards(participantAddresses[0])).to.be.revertedWith(
            'HoprStake: Nothing to claim'
          )
        })
        it('can stake some tokens', async () => {
        //   await nftContract
        //     .connect(participants[1])
        //     .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[1], stakeContract.address, 4)
          await erc677.connect(participants[1]).transferAndCall(stakeContract.address, utils.parseUnits('1.'), '0x')
        })
        it('gets the cumulated rewards right at PROGRAM_START', async function () {
          const currentAccount = await stakeContract.accounts(participantAddresses[0])
          const reward = await stakeContract.getCumulatedRewardsIncrement(participantAddresses[0])
          expect(currentAccount[2].toString()).to.equal(
            calculateRewards(1000, 0, [BASIC_FACTOR_NUMERATOR, parseInt(BADGES[0].nominator)])
          ) // equals to the expected rewards.
          expect(reward.toString()).to.equal(constants.Zero.toString()) // rewards get synced
        })
    })

    describe('Staking v2', function () {
        let rewardForClaim;
        before(async function () {
            await reset();
            await advanceTimeForNextBlock(hre.ethers.provider, PROGRAM_S2_START)
        })
        it('staking season 2 starts', async function () {
            const blockTime = await latestBlockTime(hre.ethers.provider)
            expect(blockTime.toString()).to.equal((PROGRAM_S2_START).toString())
        })
        it('gets the cumulated rewards shortly after PROGRAM_S2_START', async function () {
          await stakeContract.sync(participantAddresses[0])
          const currentAccount = await stakeContract.accounts(participantAddresses[0])
          const reward = await stakeContract.getCumulatedRewardsIncrement(participantAddresses[0])
          const blockTime = await latestBlockTime(hre.ethers.provider)
          expect(currentAccount[2].toString()).to.equal(
            calculateRewards(1000, blockTime - PROGRAM_S2_START, [
              BASIC_FACTOR_NUMERATOR,
              parseInt(BADGES[1].nominator),
              parseInt(BADGES[4].nominator)
            ])
          ) // equals to the expected rewards.
          console.log(`${currentAccount[2].toString()} AND ${reward.toString()} ${blockTime - PROGRAM_S2_START}`)
          expect(reward.toString()).to.equal(constants.Zero.toString()) // rewards get synced
          rewardForClaim = currentAccount[2].add(reward)
        })

        it('has sufficient pool', async () => {
          await expect(
            stakeContract
              .claimRewards(participantAddresses[0])
          )
            .to.emit(stakeContract, 'Claimed')
            .withArgs(participantAddresses[0], (await stakeContract.accounts(participantAddresses[0]))[3])
        })
    })

    describe('During the staking v2 program', function () {
        before(async function () {
            await reset();
            // redeem a hodler token
            await nftContract.connect(participants[1]).functions["safeTransferFrom(address,address,uint256)"](participantAddresses[1], stakeContract.address, 1);
            // mint a less good hodler token, #4
            await nftContract
            .connect(admin)
            .mint(participantAddresses[1], BADGES[3].type, BADGES[3].rank, BADGES[3].nominator, BADGES[3].deadline)
            // mint a better hodler token, #5
            await nftContract
            .connect(admin)
            .mint(participantAddresses[1], BADGES[1].type, BADGES[1].rank, BADGES[1].nominator, BADGES[1].deadline)
        })
        it('can redeem another (less good) HODLr token', async function () {
          // redeem NFT #4
          await expect(
            nftContract
              .connect(participants[1])
              .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[1], stakeContract.address, 4)
          )
            .to.emit(stakeContract, 'Redeemed')
            .withArgs(participantAddresses[1], '4', false)
        })
  
        it('can redeem another (better) HODLr token', async function () {
          // redeem token #5
          await expect(
            nftContract
              .connect(participants[1])
              .functions['safeTransferFrom(address,address,uint256)'](participantAddresses[1], stakeContract.address, 5)
          )
            .to.emit(stakeContract, 'Redeemed')
            .withArgs(participantAddresses[1], '5', true)
        })
  
        it('receives more claim rewards', async () => {
          await erc777.mintInternal(adminAddress, utils.parseUnits('5000000', 'ether'), '0x', '0x') // admin account holds 5 million wxHOPR
          await erc777.connect(admin).send(stakeContract.address, utils.parseUnits('5000000', 'ether'), '0x') // propide 5 million REWARD_TOKEN
          expect((await erc777.balanceOf(adminAddress)).toString()).to.equal(constants.Zero.toString())
        })
  
        it('cannot unlockFor tokens', async () => {
          await expect(stakeContract.unlockFor(participantAddresses[0])).to.be.revertedWith(
            'HoprStake: Program is ongoing, cannot unlock stake.'
          )
        })
  
        it('cannot unlock tokens', async () => {
          await expect(stakeContract.unlock()).to.be.revertedWith('HoprStake: Program is ongoing, cannot unlock stake.')
        })
  
        it('can reclaim random ERC20', async () => {
          const randomERC20 = await deployContractFromFactory(deployer, 'ERC20Mock', [stakeContract.address, 1])
          // Revert message was bubbled up, showing only the one from ERC677Mock
          await stakeContract.connect(admin).reclaimErc20Tokens(randomERC20.address)
          expect((await randomERC20.balanceOf(adminAddress)).toString()).to.equal('1')
        })
  
        it('can sync at anytime', async () => {
          console.log(JSON.stringify((await stakeContract.accounts(participantAddresses[0])).toString()))
          await stakeContract.sync(participantAddresses[2])
          console.log(JSON.stringify((await stakeContract.accounts(participantAddresses[0])).toString()))
        })
      })

    describe('After PROGRAM_S2_END', function () {
        before(async function () {
            await reset();

            // -----logs
            console.table([
                ["Deployer", deployerAddress],
                ["Admin", adminAddress],
                ["NFT Contract", nftContract.address],
                ["Stake Contract", stakeContract.address],
                ["participant", JSON.stringify(participantAddresses)],
            ]);
        })
        it('succeeds in advancing block to PROGRAM_S2_END + 1', async function () {
            await advanceTimeForNextBlock(hre.ethers.provider, PROGRAM_S2_END + 1);
            const blockTime = await latestBlockTime(hre.ethers.provider)
            expect(blockTime.toString()).to.equal((PROGRAM_S2_END + 1).toString()); 
        });

        it('cannot receive random 677 with `transferAndCall()`', async () => {
            // bubbled up
            await expect(erc677.connect(participants[1]).transferAndCall(stakeContract.address, constants.One, '0x')).to.be.revertedWith(
                'ERC677Mock: failed when calling onTokenTransfer'
            )
        }); 
        it('cannot redeem NFT`', async () => {
            // created #4 NFT
            await nftContract.connect(admin).mint(participantAddresses[1], BADGES[1].type, BADGES[1].rank, BADGES[1].nominator, BADGES[1].deadline);
            await expect(nftContract.connect(participants[1]).functions["safeTransferFrom(address,address,uint256)"](participantAddresses[1], stakeContract.address, 4)).to.be.revertedWith(
                'HoprStake: Program ended, cannot redeem boosts.'
            )
        }); 
        it('can unlock, and receives original tokens - Released event', async () => {
            await expect(
                stakeContract.connect(participants[0]).unlock()
              )
                .to.emit(stakeContract, 'Released')
                .withArgs(participantAddresses[0], utils.parseUnits('1000', 'ether').toString())
        }); 
        it('receives original tokens - total balance matches old one ', async () => {
            const balance = await erc677.balanceOf(participantAddresses[0]);
            expect(BigNumber.from(balance).toString()).to.equal(utils.parseUnits('10000', 'ether').toString());  // true
        }); 
        it('receives NFTs', async () => {
            const owner = await nftContract.ownerOf(0);
            expect(owner).to.equal(participantAddresses[0]); // compare bytes32 like address
        }); 
    });
});