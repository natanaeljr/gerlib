FROM ubuntu

# Basic tools
RUN apt-get update && apt-get install -y \
    curl \
    g++ \
    cmake

# Conan
RUN cd /tmp \
    && curl -LO https://dl.bintray.com/conan/installers/conan-ubuntu-64_1_18_1.deb \
    && dpkg -i conan-ubuntu-64_1_18_1.deb \
    && apt-get install -f \
    && rm conan-ubuntu-64_1_18_1.deb

# Cap'n Proto
RUN cd /tmp \
    && curl -LO https://capnproto.org/capnproto-c++-0.7.0.tar.gz \
    && tar zxf capnproto-c++-0.7.0.tar.gz \
    && cd capnproto-c++-0.7.0 \
    && ./configure \
    && make -j3 check \
    && make install \
    && cd /tmp \
    && rm -r capnproto-c++-0.7.0 capnproto-c++-0.7.0.tar.gz

# User
ARG USER_ID=1000
ARG GROUP_ID=1000

RUN groupadd -g ${GROUP_ID} duck \
    && useradd -l -u ${USER_ID} -g duck duck \
    && install -d -m 0755 -o duck -g duck /home/duck \
    && chown --changes --recursive duck:duck /home/duck

USER duck

# Workstation
VOLUME /home/duck/app
WORKDIR /home/duck/app
