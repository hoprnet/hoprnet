---
description: An introduction to Bootstrap Nodes inside HOPR Chat
---

# Bootstrap Nodes

In order to properly work, **HOPR Chat** relies in the concept of **Bootstrap Nodes**. These are nodes created with a `bootstrap` setting enabled, as to function only as relayers for other nodes in the ecosystem. Without **Bootstrap Nodes**, **HOPR Chat** will currently not work.

**Bootstrap Nodes** are only mean to be an initial relayer of connection between nodes. This means that as soon as a communication between two or more **HOPR Chat** nodes has been stablished, a **Bootstrap Node** is node longer need to keep communicating to it.

As an analogy, you can think of **Bootstrap Nodes** as the guests that know everyone in a party. They can introduce you to other people, and then you can talk directly to them.

To run **HOPR Chat** as a bootstrap node, you can just pass a `-b` flag to the running command.

## Available Bootstrap Nodes

Feel free to use any \(or all\) of the following URLs as your `BOOTSTRAP_SERVERS` parameter in your **HOPR Chat** Docker image. Each of our **Bootstrap Nodes** are located in different countries and serve a specific environment.

### 🇨🇭 Switzerland

#### Testnet Bootstrap Nodes

[![Terraform](https://github.com/hoprnet/hopr-devops/workflows/Terraform/badge.svg)](https://github.com/hoprnet/hopr-devops/workflows/Terraform/badge.svg)

**Status**: ✅ Beta

Our **Testnet Bootstrap Nodes** are fixed and deployed via our [GitHub DevOps repository](https://github.com/hoprnet/hopr-devops). These are currently being considered _beta_ technology and might be replaced at different times. However, we will notify our users when doing so via our [Telegram Channel](http://t.me/hoprnet).

{% tabs %}
{% tab title="ch-t-01" %}
```text
/dns4/ch-test-01.hoprnet.io/tcp/9091/p2p/16Uiu2HAmMUwDHzmFJaATzQPUFgzry5oxvSgWF2Vc553HCpekC4qU
```
{% endtab %}

{% tab title="ch-t-02" %}
```text
/dns4/ch-test-02.hoprnet.io/tcp/9091/p2p/16Uiu2HAmVFVHwJs7EqeRUtY6EZTtv379CiwvJgdsDfmdywbKfgAq
```
{% endtab %}
{% endtabs %}

These nodes are behind HOPR Services AG DNS registry. In case you want to directly access them without the DNS request, you can simply pass these directly.

{% tabs %}
{% tab title="ch-t-01 \(no-dns\)" %}
```text
/ip4/34.65.36.154/tcp/9091/p2p/16Uiu2HAmMUwDHzmFJaATzQPUFgzry5oxvSgWF2Vc553HCpekC4qU
```
{% endtab %}

{% tab title="ch-t-02 \(no-dns\)" %}
```
/ip4/34.65.198.231/tcp/9091/p2p/16Uiu2HAmVFVHwJs7EqeRUtY6EZTtv379CiwvJgdsDfmdywbKfgAq
```
{% endtab %}
{% endtabs %}

#### Develop Bootstrap Nodes

[![Terraform](https://github.com/hoprnet/hopr-devops/workflows/Terraform/badge.svg)](https://github.com/hoprnet/hopr-devops/workflows/Terraform/badge.svg)

**Status**: ⚠️ Alpha

Our **Develop Bootstrap Nodes** are constantly changing and deployed via our [GitHub DevOps repository](https://github.com/hoprnet/hopr-devops). These are currently being considered _alpha_ technology and are replaced multiple times over the week. To use or test them, please check our [Builds Page](https://github.com/hoprnet/hopr-devops/actions?query=workflow%3ATerraform).







