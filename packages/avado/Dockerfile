FROM gcr.io/hoprassociation/hoprd:1

VOLUME ["/app/db"]

# Admin web server
EXPOSE 3000
# REST API
EXPOSE 3001

ENTRYPOINT ["node", "/app/index.js", "--password='open-sesame-iTwnsPNg0hpagP+o6T0KOwiH9RQ0'", "--init", "--admin"]
