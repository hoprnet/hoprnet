---
id: grafana-dashboards
title: Using Grafana
---

Currently, there is only support for the Grafana dashboards on Dappnode. If you are using an Avado or VPS/Docker setup, these dashboards will become available to you on the next release: `Riga`.

The Grafana dashboards will give you an advanced look into your node, with a dashboard breaking down several key metrics about your node.

![Grafana Dashboard](/img/node/Grafana-Dashboard.png)

## Setting up Grafana Dashboard on Dappnode

To access your node's Grafana Dashboard, you need to install both:

- The latest release: `Bogota`
- The DMS Package

(**1**) First, make sure you are using the latest release, `Bogota`. Follow the instructions [here](using-dappnode#installing-the-hopr-client) to ensure you are up-to-date.

(**2**) Next, go to the DAppStore and install the package `DMS`. It stands for Dappnode Monitoring System and will give you access to your node's dashboards.

![Grafana packages](/img/node/Grafana-packages-edited.jpg)

(**3**) That's all! If your node and DMS package are running fine, you can access your Grafana dashboard by clicking the link on your DMS package.

![DMS package info](/img/node/Grafana-info-edited.jpg)

(**4**) Then under `Dashboard`, look/search for `HOPR NODE Overview`.

![Gashboard searchbar](/img/node/Grafana-dashboard-searchbar.png)

(**5**) Opening this dashboard will give you access to your node metrics.

**Note:** The image below is of a node outside the network, hence why all the graphs are flat. Your graphs should not be flat if you have a registered working node within the network.

![Initial Dashboard](/img/node/Grafana-initial-dashboard.png)
