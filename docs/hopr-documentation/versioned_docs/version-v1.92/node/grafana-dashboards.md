---
id: grafana-dashboards
title: Using Grafana
---

The Grafana dashboards will give you an advanced look into your node, with a dashboard breaking down several key metrics. You can access Grafana Dashboards on all setups:

- [Dappnode](./grafana-dashboards.md#dappnode)
- [Avado](./grafana-dashboards.md#avado) 
- [Docker](./grafana-dashboards.md#docker)

![Grafana Dashboard](/img/node/Grafana-Dashboard.png)

## Setting up Grafana Dashboard

### Dappnode

To access your node's Grafana Dashboard, you just need to install the DMS package.

(**1**) Go to the DAppStore and install the package `DMS`. It stands for Dappnode Monitoring System and will give you access to your node's dashboards.

![Grafana packages](/img/node/Grafana-packages-edited.jpg)

(**2**) That's all! If your node and DMS package are running fine, you can access your Grafana dashboard by clicking the link on your DMS package.

![DMS package info](/img/node/Grafana-info-edited.jpg)

You should get a login screen, where you'll need to use the following credentials:

- Username: `admin`
- Password: `hopr`

### Avado

For Avado, you can access the Grafana dashboards by just visiting the URL: [http://hopr.my.ava.do:3030/](http://hopr.my.ava.do:3030/). This is assuming you have HOPR installed on your Avado. You can view instructions on how to do so [here.](./using-avado.md)

You should get a login screen, where you'll need to use the following credentials:

- Username: `admin`
- Password: `hopr`

### Docker

To set up Grafana with docker you will need to have installed:

- Docker with docker-compose support, you may need to install this seperately. Read [here.](https://docs.docker.com/compose/install/)
- git E.g. through shell with `sudo apt-get install git`

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

**Note:** This will create a node with no authentication so you will not need to enter a security/API token when using it's functionality. This setup will also launch hopr-admin for you so no other commands are needed, you can proceed directly to our [hopr-admin tutorial](./using-hopr-admin.md) for further details on how to interact with your node through your browser.

Otherwise once your HOPRd node is up and running, you should have Grafana exposed on port `3030`, E.g. if you're running this locally and not on a VPS, it would be at the endpoint [http://localhost:3030](http://localhost:3030) (replace `localhost` with your `server IP address` if you are using a VPS, for example, `http://142.93.5.175:3030`).

You should get a login screen, where you'll need to use the following credentials:

- Username: `admin`
- Password: `hopr`

## Using Grafana

Once you have logged into Grafana, you can access your node's metrics by locating the dashboard: `HOPR NODE Overview`.

(**1**) Log into Grafana using the credentials:

- Username: `admin`
- Password: `hopr`

![Grafana Login](/img/node/Grafana-login.png)

(**2**) Under `Dashboard`, look/search for `HOPR NODE Overview`, under the `hopr` category.

![Gashboard searchbar](/img/node/Grafana-dashboard-searchbar.png)

(**2**) Opening this dashboard will give you access to your node metrics.

**Note:** The image below is of a node outside the network, hence why all the graphs are flat. Your graphs should not be flat if you have a registered working node within the network.

![Initial Dashboard](/img/node/Grafana-initial-dashboard.png)

Each dashboard will have a description and tooltip explaining what you are looking at for all the major and important metrics. 