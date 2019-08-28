FROM ubuntu

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

# Configure apt and install basic packages
RUN apt-get update && apt-get install -y \
    curl git sudo \
    build-essential cmake gdb cppcheck valgrind \
    # clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

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

# User Args: set with --build-arg
ARG USERNAME=duck
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Create User
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd -s /bin/bash --uid $USER_UID --gid $USER_GID --create-home $USERNAME --no-log-init \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME
USER $USERNAME

# Setup Workspace
VOLUME /home/$USERNAME/gerlib
WORKDIR /home/$USERNAME/gerlib

# Finish Conan deps installation with new user
COPY --chown=$USER_UID:$USER_GID conanfile.txt /tmp
RUN cd /tmp \
    && conan install -s build_type=Debug -s compiler.libcxx=libstdc++11 . --build=missing \
    && conan profile update settings.build_type=Debug default \
    && conan profile update settings.compiler.libcxx=libstdc++11 default \
    && rm *

# Terminal handler
ENV TERM=xterm-color

# Switch back to dialog for any ad-hoc use of apt-get
ENV DEBIAN_FRONTEND=
