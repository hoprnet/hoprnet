// TODO - addresses are hardcoded.

accounts[0]
  .getAddress()
  .then((address) =>
    new ethers.Contract(
      '0x4F88909ecc41eC032162683aa56c2384a0630f8B',
      ['function balanceOf(address owner) view returns (uint256)'],
      new ethers.providers.JsonRpcProvider('https://rpc-mainnet.matic.network')
    )
      .balanceOf(address)
      .then((b) => formatEther(b))
  )
