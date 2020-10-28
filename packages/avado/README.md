# AVADO-DNP-HOPR

## Deploy

The deploy should happen automatically through a github action.

---

## Old instructions

### Prerequisites

`sudo npm install -g @dappnode/dappnodesdk@0.2.9-beta.0`

### building the package

First connect to the VPN or Wifi of your AVADO box, then do

`dappnodesdk build`

it will give you an IPFS hash as output

Note: Usually you will develop locally using `docker-compose build --build-arg network="matic"` and `docker-compose up` untill your package works & use the building in the AVADO package format whenever you want to run the same on the AVADO box.

### Installing the package

Either install it through the DappStore (and paste in the IPFS hash there)

Note: this only works when the version has been increased - otherwise the Dappstore will not recognize it as a new version.

Or go to `http://go.ava.do/install/<IPFS hash>` to force the version on the machine - this works even if you have not updated the version number..

### updating the version

```
dappnodesdk increase patch
dappnodesdk build --provider http://23.254.227.151:5001
git add dappnode_package.json docker-compose.yml releases.json
git commit -m"new release"
git push
release-it
```

( the `23.254.227.151` server is an IPFS server we host to seed the data)
