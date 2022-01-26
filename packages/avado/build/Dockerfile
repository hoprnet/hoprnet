FROM gcr.io/hoprassociation/hoprd:athens

ENTRYPOINT [ "/usr/bin/tini", "--", "yarn", "hoprd", "--password='open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0'", "--init", "--admin", "--adminHost", "0.0.0.0", "--rest", "--restHost", "0.0.0.0", "--healthCheck", "--healthCheckHost", "0.0.0.0", "--data", "/app/db/data", "--identity", "/app/db/.hopr-identity", "--apiToken='!5qxc9Lp1BE7IFQ-nrtttU'" ]
