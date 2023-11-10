# syntax=docker/dockerfile:1.4

# This docker file runs `jest` to test the query server which is expected
# to be running at http://query-server:3000
FROM node
# RUN apk add curl

WORKDIR /code
COPY tests .

ENV QUERY_SERVER=http://query-server:3000

RUN npm install --save-dev jest

CMD ["/code/run_tests.sh"]