ARG UPSTREAM_VERSION
ARG HOPR_ADMIN_VERSION=latest
ARG PROMETHEUS_VERSION=2.41.0
ARG GRAFANA_VERSION=9.3.2

FROM gcr.io/hoprassociation/hopr-admin:${HOPR_ADMIN_VERSION} as hopr-admin

# Arguments have to be repeated for each image build stage
ARG UPSTREAM_VERSION
ARG PROMETHEUS_VERSION
ARG GRAFANA_VERSION

FROM gcr.io/hoprassociation/hoprd:${UPSTREAM_VERSION}

# Arguments have to be repeated for each image build stage
ARG PROMETHEUS_VERSION
ARG GRAFANA_VERSION

ENV AVADO=true

WORKDIR /opt
ADD https://github.com/prometheus/prometheus/releases/download/v${PROMETHEUS_VERSION}/prometheus-${PROMETHEUS_VERSION}.linux-amd64.tar.gz prometheus.tar.gz
RUN tar xzf prometheus.tar.gz && rm prometheus.tar.gz && mv prometheus* prometheus
RUN mkdir /opt/prometheus/data

ADD https://dl.grafana.com/oss/release/grafana-${GRAFANA_VERSION}.linux-amd64.tar.gz grafana.tar.gz
RUN tar xzf grafana.tar.gz && rm grafana.tar.gz && mv grafana* grafana

COPY grafana/ /etc/grafana/
COPY prometheus/ /etc/prometheus/
WORKDIR /app/hoprnet/packages/hoprd

RUN apt-get update \
 && apt-get install -y \
      pip \
 && rm -rf /var/lib/apt/lists/* \
 && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false

RUN pip install supervisor superlance

COPY supervisord.conf /etc/supervisord.conf

COPY --from=hopr-admin /app /hopr-admin

ENTRYPOINT ["supervisord", "--nodaemon", "--configuration", "/etc/supervisord.conf"]
