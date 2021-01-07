FROM gcr.io/hoprassociation/hoprd:1

VOLUME ["/app/db"]

# Admin web server
EXPOSE 3000
# REST API
EXPOSE 3001

ENTRYPOINT ["node", "/app/index.js"]
