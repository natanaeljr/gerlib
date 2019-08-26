Ger Library
===

Gerrit library written in C++.


Development
---

Development should be done under the docker container which has all needed dependencies ready.

Build docker image: `docker build --build-arg USER_ID=$(id -u) --build-arg GROUP_ID=$(id -g) --tag gerlib .`

Run docker container: `docker run -v "$PWD:/home/duck/app" -it gerlib`

