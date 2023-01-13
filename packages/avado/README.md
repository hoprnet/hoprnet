# AVADO

## Deploy

Avado images are based on the `hoprd` Docker images that can be found at [gcr.io/hoprassociation/hoprd](https://gcr.io/hoprassociation/hoprd).

The creation of the Avado image happens automatically within the `deploy` [pipeline](../../.github/workflows/deploy.yaml) using the [build-avado.sh](../../scripts/build-avado.sh) script.

## Releases

Users can install Avado images using an IPFS hash such as `/ipfs/QmNwV6i2vrLKSfJocud9g8g9SgegdXbxHJYDua4t5iZ6NV`. The latest IPFS hash can be found in [releases.json](./releases.json).
