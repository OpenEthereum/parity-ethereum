#!/bin/sh

# The image name
PARITY_IMAGE_REPO=${PARITY_IMAGE_REPO:-parity/parity}
# The tag to be used for builder image
PARITY_BUILDER_IMAGE_TAG=${PARITY_BUILDER_IMAGE_TAG:-build}
# The tag to be used for runner image
PARITY_RUNNER_IMAGE_TAG=${PARITY_RUNNER_IMAGE_TAG:-latest}

echo Building $PARITY_IMAGE_REPO:$PARITY_BUILDER_IMAGE_TAG-$(git log -1 --format="%H")
docker build --no-cache -t $PARITY_IMAGE_REPO:$PARITY_BUILDER_IMAGE_TAG-$(git log -1 --format="%H") . -f docker/centos/build.Dockerfile

echo Creating $PARITY_BUILDER_IMAGE_TAG-$(git log -1 --format="%H"), extracting binary
docker create --name extract $PARITY_IMAGE_REPO:$PARITY_BUILDER_IMAGE_TAG-$(git log -1 --format="%H") 
mkdir docker/centos/parity
docker cp extract:/build/parity-ethereum/target/release/parity docker/centos/parity

echo Building $PARITY_IMAGE_REPO:$PARITY_RUNNER_IMAGE_TAG
docker build --no-cache -t $PARITY_IMAGE_REPO:$PARITY_RUNNER_IMAGE_TAG . -f docker/centos/Dockerfile

echo Cleaning up ...
rm -rf docker/centos/parity
docker rm -f extract

echo Echoing Parity version:
docker run $PARITY_IMAGE_REPO:$PARITY_RUNNER_IMAGE_TAG --version

echo Done.
