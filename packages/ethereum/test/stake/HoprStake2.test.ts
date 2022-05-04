import * as hre from 'hardhat'
import { BigNumber, constants, Contract, Signer, utils } from 'ethers'
import{ expect } from "chai";
import deployERC1820Registry from '../../deploy/01_ERC1820Registry'
import { advanceTimeForNextBlock, deployContractFromFactory, latestBlockTime } from '../utils'

const { TEST_WHITEHAT_ONLY } = process.env;
const whitehatTestOnly = !TEST_WHITEHAT_ONLY || TEST_WHITEHAT_ONLY.toLowerCase() !== 'true'? false : true;

(whitehatTestOnly ? describe.skip : describe)('HoprStake2', function () {
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
    const PROGRAM_V2_START = 1642424400; // Jan 17 2022 14:00 CET.
    const PROGRAM_V2_END = 1650974400; // Apr 26th 2022 14:00 CET.
    const BADGES = [
        {
            type: "HODLr",
            rank: "silver",
            deadline: PROGRAM_V2_START,
            nominator: "158" // 0.5% APY
        },
        {
            type: "HODLr",
            rank: "platinum",
            deadline: PROGRAM_V2_END,
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
            deadline: PROGRAM_V2_END,
            nominator: "79" // 0.25% APY
        },
        {
            type: "Testnet participant",
            rank: "gold",
            deadline: PROGRAM_V2_END,
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

    describe('unit tests', function () {
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

    describe('After PROGRAM_V2_END', function () {
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
        it('succeeds in advancing block to PROGRAM_V2_END + 1', async function () {
            await advanceTimeForNextBlock(hre.ethers.provider, PROGRAM_V2_END + 1);
            const blockTime = await latestBlockTime(hre.ethers.provider)
            expect(blockTime.toString()).to.equal((PROGRAM_V2_END + 1).toString()); 
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