FROM debian:12-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY bin/dmtrctl-Linux-${TARGETARCH} /bin/dmtrctl
RUN chmod +x /bin/dmtrctl
RUN ln -s /bin/dmtrctl /dmtrctl

ENTRYPOINT [ "dmtrctl" ]
