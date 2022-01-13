FROM gcr.io/hoprassociation/hoprd:master-goerli@sha256:b74bdb60c471755aa533be9ec53878ba33433aec28192265d9de81d920743541

ENTRYPOINT ["/usr/bin/tini", "--", "yarn", "hoprd", "--password='open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0'", "--init", "--admin", "--adminHost", "0.0.0.0", "--identity", "/app/db/.hopr-identity", "--testNoAuthentication", "--healthCheck", "--healthCheckHost", "0.0.0.0"]
