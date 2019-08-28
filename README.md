Ger Library
===

Gerrit library written in C++.


Development
---

Development should be done under the docker container which has all needed dependencies ready.

**Build image**:

~~~shell
docker build --tag gerlib . --build-arg USER_UID=$(id -u) --build-arg USER_GID=$(id -g)
~~~

**Run container**:

~~~shell
docker run --name gerlib --rm -v "$PWD:/home/duck/gerlib" -it gerlib
~~~

**Configure and build project**:

~~~shell
cd ~/gerlib
mkdir build
cd build
conan install ..
cmake .. -DCMAKE_BUILD_TYPE=Debug -DBUILD_TESTING=ON
cmake --build .
~~~


CI/CD
---

GitLab CI/CD is used to build the docker image, build the cmake project and run all tests.  
See .gitlab-ci.yml.
