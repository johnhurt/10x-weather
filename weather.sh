#!/bin/sh

# Run this script to start the server and run the automated tests
NETWORK=10x-weather-net

# docker or finch can be used here
DOCKER=docker

set -e

# First build the images
$DOCKER build . -t 10x-weather --file server.Dockerfile 
$DOCKER build . -t 10x-weather-tester --file tester.Dockerfile

# Then create the network for them to communicate if it doesn't already exist
$DOCKER network inspect $NETWORK > /dev/null 2>&1 \
    || $DOCKER network create --driver=bridge $NETWORK 

# Start the server on the created network if it doesn't already exist

{ $DOCKER run --rm --name query-server -p 3000:3000 --network=$NETWORK 10x-weather & } 2>&1 > /dev/null

echo Running test suite

# Now run the automated tests (These will wait for a while for the server to start)
$DOCKER run --rm --network=$NETWORK 10x-weather-tester

# Stop the server 
$DOCKER kill --signal="SIGINT" $($DOCKER ps -a | grep ten-x-weather | cut -d" " -f1)

echo Starting the server on http://localhost:3000. Have fun!

# Restart the server on the host network 
$DOCKER run --rm -p 3000:3000 10x-weather 