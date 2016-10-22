#!/bin/bash
set -e

# setup the git user defaults for the current repo
function setup_git_user {
  git config push.default simple
  git config merge.ours.driver true
  git config user.email "jaco+gitlab@ethcore.io"
  git config user.name "GitLab Build Bot"
}

# change into the build directory
pushd `dirname $0`
cd ../.build

# variables
UTCDATE=`date -u "+%Y%m%d-%H%M%S"`

# Create proper directory structure
mkdir -p build
mv * build || true
mkdir -p src

# Copy rust files
cp ../Cargo.precompiled.toml Cargo.toml
cp ../build.rs .
cp ../src/lib.rs* ./src/

# init git
rm -rf ./.git
git init

# add local files and send it up
setup_git_user
git remote add origin https://${GITHUB_JS_PRECOMPILED}:@github.com/ethcore/js-precompiled.git
git fetch origin
git checkout -b $CI_BUILD_REF_NAME
git add .
git commit -m "$UTCDATE [compiled]"
git merge origin/$CI_BUILD_REF_NAME -X ours --commit -m "$UTCDATE [release]"
git push origin $CI_BUILD_REF_NAME

# back to root
popd

# bump js-precompiled
cargo update -p parity-ui-precompiled

# add to git and push
setup_git_user
git remote add origin https://${GITHUB_JS_PRECOMPILED}:@github.com/ethcore/parity.git
git fetch origin
git add .
git commit -m "[ci skip] js-precompiled $UTCDATE"
git push origin

# exit with exit code
exit 0
