#!/bin/bash

cd -- "$(dirname "$0")"
ls -d "$PWD/prebuilds/darwin-x64/"* | xargs -I {} xattr -r -d com.apple.quarantine {}
ls -d "$PWD/build/Release/"* | xargs -I {} xattr -r -d com.apple.quarantine {}
node index.js -p switzerland 2>log.txt