#!/bin/bash
set -e
IMAGE_NAME="${IMAGE_NAME:-totp}"
TAG="${TAG:-latest}"
DOCKER_BUILDKIT=1 docker build -t "$IMAGE_NAME:$TAG" --progress=plain .

if [ -n "$SSH_HOST" ]; then
  echo "Deploying to $SSH_HOST"
  docker save "$IMAGE_NAME:$TAG" | bzip2 | pv | ssh -o 'RemoteCommand=none' "$SSH_HOST"  'bunzip2 | docker load'
else
  # shellcheck disable=SC2016
  echo 'Set $SSH_HOST to automatically deploy'
fi
