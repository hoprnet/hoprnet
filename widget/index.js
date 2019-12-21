// store elements and avoid multiple calls to dom
const elements = new Map();

// get element and update cache
const getElementById = id => {
  if (elements.has(id)) return elements.get(id);

  const element = document.getElementById(id);
  if (element) elements.set(id, element);

  return element;
};

const changeTab = name => {
  const views = {
    stats: {
      container: getElementById("stats-container"),
      tab: getElementById("stats-tab")
    },
    stake: {
      container: getElementById("stake-container"),
      tab: getElementById("stake-tab")
    },
    votes: {
      container: getElementById("votes-container"),
      tab: getElementById("votes-tab")
    }
  };

  // loop through views and update the styles
  for (const tabName in views) {
    const { container, tab } = views[tabName];

    if (tabName === name) {
      container.classList.remove("hidden");
      tab.classList.add("tab-active");
    } else {
      container.classList.add("hidden");
      tab.classList.remove("tab-active");
    }
  }
};

function toggleTheme() {
  // swap colors
  getElementById("html").classList.toggle("theme-light");
  getElementById("html").classList.toggle("theme-dark");

  // swap visibility
  getElementById("sun").classList.toggle("hidden");
  getElementById("moon").classList.toggle("hidden");
}

const onTabClick = changeTab;
const onToggleTheme = toggleTheme;

// stake
const stake = async () => {
  const $from = getElementById("stake-from");
  const $to = getElementById("stake-to");
  const $amount = getElementById("stake-amount");

  // const token = new window.web3.eth.Contract(TOKEN_ABI, TOKEN_ADDRESS);
  const paymentChannel = new window.web3.eth.Contract(
    PAYMENT_CHANNEL_ABI,
    PAYMENT_CHANNEL_ADDRESS
  );

  const [funder] = await window.web3.eth.getAccounts();
  const sender = $from.value;
  const recipient = $to.value;
  const amount = window.web3.utils.toWei(String($amount.value) || "0", "ether");
  const oneMonth = 60 * 60 * 24 * 31;

  await enableWeb3();

  return paymentChannel.methods
    .createChannel(funder, sender, recipient, TOKEN_ADDRESS, amount, oneMonth)
    .send({ from: funder })
    .on("transactionHash", hash => {
      console.log("transactionHash", hash);
    })
    .on("confirmation", confirmationNumber => {
      console.log("confirmationNumber", confirmationNumber);
    })
    .on("error", console.error);
};
const onStake = stake;

// withdraw
const withdraw = async channel => {
  const [recipient] = await window.web3.eth.getAccounts();
  const signature = await signPayment(
    recipient,
    PAYMENT_CHANNEL_ADDRESS,
    channel.deposit
  );

  const paymentChannel = new window.web3.eth.Contract(
    PAYMENT_CHANNEL_ABI,
    PAYMENT_CHANNEL_ADDRESS
  );

  await enableWeb3();

  return paymentChannel.methods
    .closeChannel(channel.id, channel.deposit, signature)
    .send({ from: recipient })
    .on("transactionHash", hash => {
      console.log("transactionHash", hash);
    })
    .on("confirmation", confirmationNumber => {
      console.log("confirmationNumber", confirmationNumber);
    })
    .on("error", console.error);
};

// render user data
const renderUserData = async () => {
  const token = new window.web3.eth.Contract(TOKEN_ABI, TOKEN_ADDRESS);

  const [userAddress] = await window.web3.eth.getAccounts();
  const tokenBalance = await token.methods
    .balanceOf(userAddress)
    .call()
    .then(res => window.web3.utils.fromWei(res || "0", "ether"));

  const $address = getElementById("wallet-info-address");
  const $balance = getElementById("wallet-info-tokenBalance");

  $address.innerText = `Your Wallet Address: ${userAddress}`;
  $balance.innerText = `Balance: ${tokenBalance} HOPR`;
};

const renderStatsRow = async (channel, event) => {
  const $table = getElementById("table-stats");
  const $rowInDom = getElementById(`table-stats-${channel.id}`);

  const isNew = !$rowInDom;
  const $row = isNew ? $table.insertRow(-1) : $rowInDom;
  const $from = isNew ? $row.insertCell(0) : $row[0];
  const $to = isNew ? $row.insertCell(1) : $row[1];
  const $amount = isNew ? $row.insertCell(2) : $row[2];
  const $opened = isNew ? $row.insertCell(3) : $row[3];
  const $status = isNew ? $row.insertCell(4) : $row[4];

  $row.id = `table-stats-${channel.id}`;

  $from.innerHTML = `
    <a
      href="https://etherscan.io/address/${channel.sender}"
      target="_blank"
      rel="noopener noreferrer"
      >${tinyAddress(channel.sender)}</a
    > 
  `;

  $to.innerHTML = `
    <a
      href="https://etherscan.io/address/${channel.recipient}"
      target="_blank"
      rel="noopener noreferrer"
      >${tinyAddress(channel.recipient)}</a
    > 
  `;

  $amount.innerText = window.web3.utils.fromWei(channel.deposit, "ether");

  const timestamp = await window.web3.eth
    .getBlock(event.blockNumber)
    .then(res => Number(res.timestamp) * 1e3);
  $opened.innerText = new Date(timestamp);

  $status.innerText = channel.status;
};

const renderStakeRow = async (channel, event) => {
  const [userAddress] = await window.web3.eth.getAccounts();

  const $table = getElementById("table-stake");
  const $rowInDom = getElementById(`table-stake-${channel.id}`);

  const isNew = !$rowInDom;
  const $row = isNew ? $table.insertRow(-1) : $rowInDom;
  const $from = isNew ? $row.insertCell(0) : $row[0];
  const $to = isNew ? $row.insertCell(1) : $row[1];
  const $amount = isNew ? $row.insertCell(2) : $row[2];
  const $opened = isNew ? $row.insertCell(3) : $row[3];
  const $status = isNew ? $row.insertCell(4) : $row[4];

  $row.id = `table-stake-${channel.id}`;

  $from.innerHTML = `
    <a
      href="https://etherscan.io/address/${channel.sender}"
      target="_blank"
      rel="noopener noreferrer"
      >${tinyAddress(channel.sender)}</a
    > 
  `;

  $to.innerHTML = `
    <a
      href="https://etherscan.io/address/${channel.recipient}"
      target="_blank"
      rel="noopener noreferrer"
      >${tinyAddress(channel.recipient)}</a
    > 
  `;

  $amount.innerText = window.web3.utils.fromWei(channel.deposit, "ether");

  const timestamp = await window.web3.eth
    .getBlock(event.blockNumber)
    .then(res => Number(res.timestamp) * 1e3);
  $opened.innerText = new Date(timestamp);

  const isRecipient = channel.recipient === userAddress;
  const isOpened = channel.status === "OPEN";

  if (isRecipient && isOpened) {
    $status.classList.add("table-button");
    $status.innerText = "WITHDRAW";
    $status.onclick = () => withdraw(channel);
  } else {
    $status.classList.remove("table-button");
    $status.innerText = channel.status;
  }
};

window.addEventListener("load", async () => {
  const [userAddress] = await window.web3.eth.getAccounts();

  const token = new window.web3.eth.Contract(TOKEN_ABI, TOKEN_ADDRESS);
  console.log("token", token);
  const paymentChannel = new window.web3.eth.Contract(
    PAYMENT_CHANNEL_ABI,
    PAYMENT_CHANNEL_ADDRESS
  );

  // get user data
  renderUserData();

  // get past events & watch for new events
  const channels = new Map();

  const processEvent = async event => {
    const data = event.returnValues;
    const isNew = !channels.has(data.id) && event.event === "OpenedChannel";

    if (isNew) {
      channels.set(data.id, {
        ...data,
        status: "OPEN"
      });
    } else if (event.event === "ClosedChannel") {
      const channel = await paymentChannel.methods.channels(data.id).call();

      channels.set(data.id, {
        ...channel,
        ...data,
        status: "CLOSE"
      });
    }

    const newChannel = channels.get(data.id);

    return Promise.all([
      renderStatsRow(newChannel, event),
      data.recipient === userAddress
        ? renderStakeRow(newChannel, event)
        : Promise.resolve()
    ]);
  };

  const getEvents = async () => {
    const openedEvents = await paymentChannel.getPastEvents("OpenedChannel", {
      fromBlock: 0
    });
    await Promise.all(openedEvents.map(event => processEvent(event)));

    const closedEvents = await paymentChannel.getPastEvents("ClosedChannel", {
      fromBlock: 0
    });
    await Promise.all(closedEvents.map(event => processEvent(event)));
  };

  setInterval(getEvents, 1e3);
});

const tinyAddress = address => {
  return `${address.slice(0, 5)}...`;
};

const createMessage = (contract, amount) => {
  return window.web3.utils.soliditySha3(
    { type: "address", value: contract },
    { type: "uint256", value: amount }
  );
};

const signMessage = (signer, messageHash) => {
  return web3.eth.personal.sign(messageHash, signer);
};

const signPayment = (signer, contract, amount) => {
  const messageHash = createMessage(contract, amount);

  return signMessage(signer, messageHash);
};
