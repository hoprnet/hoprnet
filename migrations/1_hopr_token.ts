const HoprToken = artifacts.require("HoprToken");

module.exports = (async (deployer, network, [owner]) => {
  await deployer.deploy(HoprToken);

  if (network === "development") {
    const hoprToken = await HoprToken.deployed();

    // mint tokens for owner
    await hoprToken.mint(owner, web3.utils.toWei("100", "ether"), {
      from: owner
    });
  }
}) as Truffle.Migration;
