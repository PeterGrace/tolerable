FROM docker.io/rust:1-buster as build
ARG TARGETARCH
ENV TARGETARCH=${TARGETARCH:-amd64}
ADD . /src
WORKDIR /src
RUN cargo build --release --target $(/src/tools/target_arch.sh) \
 && mv /src/target/$(/src/tools/target_arch.sh)/release/tolerable /tolerable

FROM docker.io/debian:buster

RUN mkdir -p /opt/tolerable 
WORKDIR /opt/tolerable
COPY ./tools/target_arch.sh /opt/tolerable
COPY --from=build /tolerable /opt/tolerable/tolerable
COPY docker/tolerable.toml /opt/tolerable/
CMD ["/opt/tolerable/tolerable"]
EXPOSE 8443
