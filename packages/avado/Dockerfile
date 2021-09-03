FROM gcr.io/hoprassociation/hoprd:latest

ENTRYPOINT ["/usr/bin/tini", "--", "yarn", "hoprd", "--password='open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0'", "--init", "--admin", "--adminHost", "0.0.0.0", "--identity", "/app/db/.hopr-identity", "--testNoAuthentication", "--healthCheck", "--healthCheckHost", "0.0.0.0"]
