#!/usr/bin/env bash

echo "Announce changes to the world!"

VERSION=$(tomlq -r '.package.version' Cargo.toml)
echo "Version: $VERSION"

# Send announcement to a standalone test Slack channel.
docker run \
   --rm \
  -v $PWD:/announcer \
  metaswitch/announcer:2.3.0 \
    announce \
    --slackhook $SLACK_HOOK \
    --changelogversion $VERSION \
    --changelogfile /announcer/CHANGELOG.md \
    --projectname announcer \
    --username travis-announcer \
    --iconemoji party_parrot

