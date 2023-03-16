FROM docker.io/ubuntu:20.04
ARG TARGETARCH

RUN mkdir -p /opt/tolerable 
WORKDIR /opt/tolerable
COPY ./tools/target_arch.sh /opt/tolerable
COPY docker/tolerable.toml /opt/tolerable/
RUN --mount=type=bind,target=/context \
 cp /context/target/$(/opt/tolerable/target_arch.sh)/release/tolerable /opt/tolerable/tolerable
CMD ["/opt/tolerable/tolerable"]
EXPOSE 8443
