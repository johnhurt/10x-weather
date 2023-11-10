#!/bin/sh
attempt_counter=0
max_attempts=60

# Wait for the server to start
until $(curl --output /dev/null --silent --head --fail $QUERY_SERVER); do
    if [ ${attempt_counter} -eq ${max_attempts} ];then
      echo "Max attempts reached"
      exit 1
    fi

    printf '.'
    attempt_counter=$(($attempt_counter+1))
    sleep 1
done

npm test