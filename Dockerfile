FROM docker.io/ubuntu:20.04
ARG TARGETARCH
ENV TARGETARCH=${TARGETARCH:-amd64}

RUN mkdir -p /opt/tolerable /opt/target
COPY ./tools/target_arch.sh ./target_arch.sh
COPY ./target /opt/target
RUN cp /opt/target/$(./target_arch.sh)/release/tolerable /opt/tolerable \
 && rm -rf /opt/target
WORKDIR /opt/tolerable
COPY docker/tolerable.toml /opt/tolerable/
CMD ["/opt/tolerable/tolerable"]
EXPOSE 8443
