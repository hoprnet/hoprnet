---
id: grafana-dashboards
title: Using Grafana
---

The Grafana dashboards will give you an advanced look into your node, with a dashboard breaking down several key metrics. You can access Grafana Dashboards on all setups:

- ![Dappnode](./grafana-dashboards.md#dappnode)
- ![Avado](./grafana-dashboards.md#avado) 
- ![Docker](./grafana-dashboards.md#docker)

![Grafana Dashboard](/img/node/Grafana-Dashboard.png)

## Setting up Grafana Dashboard

### Dappnode

To access your node's Grafana Dashboard, you just need to install the DMS package.

(**1**) Go to the DAppStore and install the package `DMS`. It stands for Dappnode Monitoring System and will give you access to your node's dashboards.

![Grafana packages](/img/node/Grafana-packages-edited.jpg)

(**2**) That's all! If your node and DMS package are running fine, you can access your Grafana dashboard by clicking the link on your DMS package.

![DMS package info](/img/node/Grafana-info-edited.jpg)

You should get a login screen, where you'll need to use the following credentials:

- `Username`: `admin`
- `password`: `hopr`

![Grafana Login](/img/node/Grafana-login.png)

### Avado

For Avado, you can access the Grafana dashboards by just visiting the URL: [http://hopr.my.ava.do:3030/](http://hopr.my.ava.do:3030/). This is assuming you have HOPR installed on your Avado. You can view instructions on how to do so [here.](./using-avado.md)

You should get a login screen, where you'll need to use the following credentials:

- `Username`: `admin`
- `password`: `hopr`

![Grafana Login](/img/node/Grafana-login.png)

### Docker

You'll need to have docker with docker-compose support; you may need to install this separately. You can read up on how to do so [here.](https://docs.docker.com/compose/install/)

The following are command line instructions for Linux:

(**1**) Clone the hoprnet monorepo using the following command

```bash
git clone https://github.com/hoprnet/hoprnet.git
```

(**2**) Enter the downloaded file 

```bash
cd hoprnet
```

(**3**) Run the following command to start up a HOPRd node with monitoring capabilities:

```bash
docker compose --file scripts/compose/docker-compose.yml up -d
```

**Note:** Generating your node this way will give it the default api/secret token: `YOUR_SECURITY_TOKEN`. You will need this api token to access your node.

Once your HOPRd node is up and running, you should have Grafana exposed on port `3030`, E.g. if you're running this locally and not on a VPS, it would be at the endpoint [http://localhost:3000](http://localhost:3000) (replace `localhost` with your `server IP address` if you are using a VPS, for example, `http://142.93.5.175:3000`).

You should get a login screen, where you'll need to use the following credentials:

- `Username`: `admin`
- `password`: `hopr`

![Grafana Login](/img/node/Grafana-login.png)

## Using Grafana

Once you have logged into Grafana, you can access your node's metrics by locating the dashboard: `HOPR NODE Overview`.

(**1**) Under `Dashboard`, look/search for `HOPR NODE Overview`, under the `hopr` category.

![Gashboard searchbar](/img/node/Grafana-dashboard-searchbar.png)

(**2**) Opening this dashboard will give you access to your node metrics.

**Note:** The image below is of a node outside the network, hence why all the graphs are flat. Your graphs should not be flat if you have a registered working node within the network.

![Initial Dashboard](/img/node/Grafana-initial-dashboard.png)

Each dashboard will have a description and tooltip explaining what you are looking at for all the major and important metrics. 

## Heartbeat

Many of your Grafana metrics will relate to pings generated from the `heartbeat` of your node or other nodes on the network. The heartbeat mechanism is simply the periodic pings every node on the network sends to its visible peers.

Your node will send a ping to every node it can see and record whether or not it gets a response. If there is no response, it will wait twice as long to send the next ping. This exponential backoff starts with the default of one ping every two seconds and doubles until 512 seconds, the maximum delay between pings to a specific node. 

When a successful ping goes through, the pinging delay reduces to two seconds again. A node's ratio of successful pings to total pings generates the [connectivity score](./hoprd-commands.md#info) between those two nodes.