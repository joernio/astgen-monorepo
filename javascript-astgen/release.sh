#!/usr/bin/env bash

npm version patch
git push --tags origin main
npm publish --access=public
