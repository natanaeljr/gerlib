Ger Library
===

Gerrit library written in C++.


Development
---

Development should be done under the docker container which has all needed dependencies ready.

Build docker image: `docker build --build-arg USER_ID=$(id -u) --build-arg GROUP_ID=$(id -g) --tag gerlib .`

Run docker container: `docker run -v "$PWD:/home/duck/app" -it gerlib`


CI/CD
---

GitLab CI/CD is used to build the docker image, build the cmake project and run all tests.
This is done for every commit. See .gitlab-ci.yml.
