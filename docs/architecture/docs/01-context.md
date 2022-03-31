## Continuous Deployment

Our continuous deployment pipeline is built using Github Actions.
Most logic is encapsuled in the Workflow: Deploy.

### Funding Nodes

An essential step of deployment is funding the on-chain accounts of the nodes
which have been started. This may be done in parallel for multiple nodes,
depending on how often the workflow is run in parallel.

Therefore, all funding calls are handled by a central API, which acts as a
hot-wallet. It tracks nonce-use and processes funding requests sequentially.

This setup prevents race-conditions between multiple funding calls to lead to
perpetually failing funding calls.

![](embed:gh-funding)
