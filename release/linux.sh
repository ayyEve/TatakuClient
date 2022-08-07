#!/bin/bash

# get the client version
CLIENT_VERSION="$(cat ./.client)"

# build the url
URL="https://releases.tataku.ca/${CI_BRANCH_NAME}/${CLIENT_VERSION}"

# send off
curl \
    -X PUT \
    -F "token=$TATAKU_RELEASE_KEY" \
    -F "filename=tataku-client" \
    -F "file=@./target/release/tataku-client" \
    $URL