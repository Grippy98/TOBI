FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV LANG=en_US.UTF-8
ENV LC_ALL=en_US.UTF-8

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        bc \
        bison \
        bzip2 \
        ca-certificates \
        chrpath \
        cpio \
        curl \
        diffstat \
        file \
        flex \
        gawk \
        gcc \
        g++ \
        git \
        iputils-ping \
        libegl1-mesa \
        liblz4-tool \
        libssl-dev \
        locales \
        make \
        mesa-common-dev \
        patch \
        python3 \
        python3-git \
        python3-jinja2 \
        python3-pexpect \
        python3-pip \
        python3-subunit \
        rsync \
        socat \
        tar \
        texinfo \
        unzip \
        wget \
        xz-utils \
        zstd \
    && locale-gen en_US.UTF-8 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

CMD ["bash"]
