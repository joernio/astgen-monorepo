#!/bin/bash

# Build the project
go build ./...

# Check if the build was successful
if [ $? -eq 0 ]; then
  echo "Build successful"

  # Run unit tests
  go test ./...
else
  echo "Build failed"
fi