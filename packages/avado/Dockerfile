FROM gcr.io/hoprassociation/hoprd:latest@sha256:06b3f69a28368428e361a09e14f965e958aad060a7fe9732cab6704c7bf91bce

ENTRYPOINT ["/usr/bin/tini", "--", "yarn", "hoprd", "--password='open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0'", "--init", "--admin", "--adminHost", "0.0.0.0", "--identity", "/app/db/.hopr-identity", "--testNoAuthentication", "--healthCheck", "--healthCheckHost", "0.0.0.0"]
