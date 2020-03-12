# Website of HOPR
[![Netlify Status](https://api.netlify.com/api/v1/badges/2a7471f3-7f36-48de-ad74-a12b351d5d80/deploy-status)](https://app.netlify.com/sites/hoprnet/deploys)
## Hosting
The website is hosted in a serverless fashion on [Google App Engine](https://cloud.google.com/appengine/), a simple tutorial can be found in their [documentation](https://cloud.google.com/appengine/docs/standard/python/getting-started/hosting-a-static-website). The basic settings are configured in the `app.yaml` file.

1. Register for [Google Cloud Platform](https://console.cloud.google.com/) and create a new project
2. [Install `gcloud`](https://cloud.google.com/sdk/docs) command line tools
3. Initialize via `gcloud init`, and select the GCP project that you created in step 1
4. Clone this repository and navigate into the root folder of this repository on your local machine
5. Deploy the app via `gcloud app deploy`, the first time this creates an app and lets you choose the region in which to deploy the app
6. View the page via `gcloud app browse` which should open a browser window on a domain called `YOURAPPID.appspot.com`

## Deployment
Our website is configured to auto-deploy via GitHub. Following the [Google Cloud Build documentation](https://cloud.google.com/cloud-build/docs/automating-builds/run-builds-on-github#installing_the_google_cloud_build_app), enable the Cloud Build API (just the first confirmation is enough, you do not need to check the API key) and then head to GitHub Marketplace to add the [Google Cloud Build app](https://github.com/marketplace/google-cloud-build) to your GitHub repository. After adding the Google Cloud Build app on GitHub, the installation will take you back to GCP to select the project (e.g. the one created in the previous section), consent to storing GitHub login information, select the repository and create the push trigger.

Open [Google Cloud Build](https://console.cloud.google.com/cloud-build/) in GCP, you should only see one build that gets triggered for the installation of the toolchain. Now do some modification in the repository, commit and push the changes to GitHub. You should within 15s see that a new build got started on Google Cloud Build.

After configuring the above, any commit to the repository that gets pushed to GitHub will initiate a build via `gcloud app deploy` which is triggered via the `cloudbuild.yaml` file.

You might have to change the [IAM permissions](https://console.cloud.google.com/iam-admin) for the account `PROJECTID@cloudbuild.gserviceaccount.com` to Project Owner. **TODO**: find out what permissions are actually needed as this permission is maximally wide.
