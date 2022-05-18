import { HardhatRuntimeEnvironment } from "hardhat/types";
import { DeployFunction } from "hardhat-deploy/types";
import type { HoprBoost } from '../src/types'

const DEV_NFT_TYPE = 'Dev';
const DEV_NFT_BOOST = 0;
const NUM_DEV_NFT = 6;
const DUMMY_NFT_TYPE = 'Dummy';
const DUMMY_NFT_BOOST = 10;

const main: DeployFunction = async function ({ ethers, deployments, getNamedAccounts }: HardhatRuntimeEnvironment) {
    const { utils } = ethers;
    const admin = await getNamedAccounts().then((o) => ethers.getSigner(o.admin))

    // check boost types being created
    const stakeDeployment = await deployments.get("HoprStake")
    const boostDeployment = await deployments.get("HoprBoost")
    const hoprBoost = (await ethers.getContractFactory('HoprBoost')).attach(boostDeployment.address) as HoprBoost

    // find the blocked NFT types and check the created boost types
    const blockedNftTypes = stakeDeployment.receipt.logs.filter(log => 
        log.topics[0] === utils.keccak256(utils.toUtf8Bytes("NftBlocked(uint256)"))
    ).map(log => parseInt(log.topics[1]))

    // get max blocked nft type index
    const blockNftTypeMax = blockedNftTypes.reduce((a, b) => Math.max(a,b));
    // get nft types created in the HoprBoost contract
    let devNftIndex = null;
    let loopCompleted = false;
    let index = 0;
    // loop through the array storage and record the length and dev nft index, if any
    while (!loopCompleted) {
        try {
            const createdNftTypes = await hoprBoost.typeAt(index + 1);  // array of types are 1-based
            console.log(`createdNftTypes ${createdNftTypes}`)
            if (createdNftTypes === DEV_NFT_TYPE) {
                devNftIndex = index;
            }
        } catch (error) {
            // reaching the end of nft index array storage: panic code 0x32 (Array accessed at an out-of-bounds or negative index
            if (`${error}`.match(/0x32/g)) {
                loopCompleted = true
            } else {
                console.log(`Error in checking HoprBoost types. ${error}`)
            }
        }        
        index ++;
    }
    // assign the dev nft if dev nft does not exist.
    if (!devNftIndex) {
        devNftIndex = Math.max(blockNftTypeMax, index) + 1;
    }

    console.log(`HoprBoost NFT now has ${index - 1} types and should have at least ${blockNftTypeMax} types, where ${blockedNftTypes} are blocked. Dev NFT should be at ${devNftIndex}`)

    // mint all the dummy NFTs (especially those are blocked in the constructor). Mint some dev NFTs
    while(index <= blockNftTypeMax || index <= devNftIndex) {
        if (index === devNftIndex) {
            await hoprBoost.connect(admin).batchMint(new Array(NUM_DEV_NFT).fill(admin.address), DEV_NFT_TYPE, DEV_NFT_TYPE, DEV_NFT_BOOST, 0);
        } else {
            await hoprBoost.connect(admin).mint(admin.address, `${DUMMY_NFT_TYPE}_${index}`, DUMMY_NFT_TYPE, DUMMY_NFT_BOOST, 0);
        }
        index ++;
    }

    console.log(`Admin ${admin.address} has ${await hoprBoost.balanceOf(admin.address)} Boost NFTs`)
};

main.skip = async (env: HardhatRuntimeEnvironment) => !!env.network.tags.production

export default main;
