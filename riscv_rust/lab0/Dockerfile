FROM rust-os-camp-2022:latest

RUN sed -i 's/archive.ubuntu.com/mirrors.ustc.edu.cn/g' /etc/apt/sources.list
RUN sed -i 's/security.ubuntu.com/mirrors.ustc.edu.cn/g' /etc/apt/sources.list


RUN apt-get update
RUN apt-get install -y --fix-missing --no-install-recommends vim tzdata


RUN ls -sf /usr/share/zoneinfo/Asia/Shanghai /etc/timezone && echo 'Asia/Shanghai' > /etc/timezone

RUN mkdir -p /root/.cargo/

COPY config /root/.cargo/

WORKDIR /mnt
