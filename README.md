# Website of HOPR

## Hosting
The website is hosted in a serverless fashion on [Google App Engine](https://cloud.google.com/appengine/), a simple tutorial can be found in their [documentation](https://cloud.google.com/appengine/docs/standard/python/getting-started/hosting-a-static-website). The basic settings are configured in the `app.yaml` file.

## Deployment
Our website is configured to auto-deploy via GitHub. Following the [Google Cloud Build documentation](https://cloud.google.com/cloud-build/docs/automating-builds/run-builds-on-github#installing_the_google_cloud_build_app), enable the Cloud Build API and then head to GitHub Marketplace to add the [Google Cloud Build app](https://github.com/marketplace/google-cloud-build) to your GitHub repository.

After configuring the above, any commit to the repository that gets pushed to GitHub will initiate a build via `gcloud app deploy` which is triggered via the `cloudbuild.yaml` file.

For building enable API access:
https://console.developers.google.com/apis/api/appengine.googleapis.com/overview?project=721594976025
