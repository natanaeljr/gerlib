FROM ubuntu

# Install basic tools
RUN apt-get update && apt-get install -y \
    sudo \
    curl \
    git \
    g++ \
    gdb \
    cmake

# Install Conan
RUN cd /tmp \
    && curl -LO https://dl.bintray.com/conan/installers/conan-ubuntu-64_1_18_1.deb \
    && dpkg -i conan-ubuntu-64_1_18_1.deb \
    && apt-get install -f \
    && rm conan-ubuntu-64_1_18_1.deb

# Install Cap'n Proto
RUN cd /tmp \
    && curl -L https://capnproto.org/capnproto-c++-0.7.0.tar.gz | tar zx \
    && cd capnproto-c++-0.7.0 \
    && ./configure CXXFLAGS="-DHOLES_NOT_SUPPORTED=1" \
    && make -j3 check \
    && make install \
    && cd /tmp \
    && rm -r capnproto-c++-0.7.0 \
    && mv /usr/local/include/capnp/json.capnp /usr/local/include/capnp/compat

# Install FMT 6
RUN cd /tmp \
    && curl -L https://github.com/fmtlib/fmt/archive/6.0.0.tar.gz | tar zx \
    && cd fmt-6.0.0 \
    && cmake . -DCMAKE_BUILD_TYPE=Release -DFMT_DOC=OFF -DFMT_TEST=OFF \
    && make install \
    && cd /tmp \
    && rm -r fmt-6.0.0

# Setup Workstation
VOLUME /home/duck/gerlib
WORKDIR /home/duck/gerlib

# Create User
ARG USER_ID=1000
ARG GROUP_ID=1000

RUN groupadd -g ${GROUP_ID} duck \
    && useradd -l -u ${USER_ID} -g duck duck \
    && echo "duck:duck" | chpasswd \
    && install -d -m 0755 -o duck -g duck /home/duck \
    && chown --changes --recursive duck:duck /home/duck \
    && adduser duck sudo

USER duck

# Finish Conan deps installation with new user
COPY --chown=${USER_ID}:${GROUP_ID} conanfile.txt /tmp

RUN cd /tmp \
    && conan install -s build_type=Debug -s compiler.libcxx=libstdc++11 . --build=missing \
    && conan profile update settings.build_type=Debug default \
    && conan profile update settings.compiler.libcxx=libstdc++11 default \
    && rm *
