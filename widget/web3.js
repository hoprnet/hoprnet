const enableWeb3 = async () => {
  if (window.ethereum) {
    try {
      await ethereum.enable();
    } catch (error) {
      console.error(error);
    }
  }
};

const initWeb3 = async () => {
  // Modern dapp browsers...
  if (window.ethereum) {
    window.web3 = new Web3(ethereum);
    const [initialAccount] = await window.web3.eth.getAccounts();

    window.ethereum
      .on("chainChanged", () => {
        document.location.reload();
      })
      .on("accountsChanged", ([newAccount]) => {
        if (initialAccount !== newAccount) document.location.reload();
      });
  }
  // Legacy dapp browsers...
  else if (window.web3) {
    window.web3 = new Web3(web3.currentProvider);
  }
  // Non-dapp browsers...
  else {
    console.log(
      "Non-Ethereum browser detected. You should consider trying MetaMask!"
    );
  }
};

initWeb3();
