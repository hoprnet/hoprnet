workspace "HOPRd" "Architecture and CI/CD documentation for all HOPRd-related topics." {
    !docs docs

    model {
        githubActions = softwareSystem "Github Actions" {
            deploy1 = container "Workflow: Deploy (Job 1)"
            deploy2 = container "Workflow: Deploy (Job 2)"
            deploy3 = container "Workflow: Deploy (Job 3)"
        }

        hoprd = softwareSystem "hoprd" {
            hoprdIdentity = container "hoprd identity account"
            hoprdIdentity1 = container "hoprd identity account 1"
            hoprdIdentity2 = container "hoprd identity account 2"
            hoprdIdentity3 = container "hoprd identity account 3"
        }

        api = softwareSystem "API" {
        }

        gcloud = deploymentEnvironment "Google Cloud Engine" {
            deploymentNode "master-goerli" {
                deploymentNode "master-goerli-1" {
                    master_goerli_1_id = containerInstance hoprdIdentity
                }
                deploymentNode "master-goerli-2" {
                    master_goerli_2_id = containerInstance hoprdIdentity
                    }
                deploymentNode "master-goerli-3" {
                    master_goerli_3_id = containerInstance hoprdIdentity
                    }
            }
            deploymentNode "release-a" {
                deploymentNode "release-a-1"
                deploymentNode "release-a-2"
                deploymentNode "release-a-3"
            }
            deploymentNode "release-b" {
                deploymentNode "release-b-1"
                deploymentNode "release-b-2"
                deploymentNode "release-b-3"
            }
        }
    }

    views {
        dynamic githubActions "gh-funding" {
            title "Funding process during multiple Github Actions workflow executions"

            deploy1 -> api "Requests funds for account 1"
            deploy2 -> api "Requests funds for account 1"
            deploy3 -> api "Requests funds for account 1"

            api -> hoprdIdentity1 "Funds on-chain account"
            api -> deploy1 "Returns ok or error"

            api -> hoprdIdentity2 "Funds on-chain account"
            api -> deploy2 "Returns ok or error"

            api -> hoprdIdentity3 "Funds on-chain account"
            api -> deploy3 "Returns ok or error"

            autoLayout lr
        }

        theme default
    }
}
