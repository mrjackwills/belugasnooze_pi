#########
# SETUP #
#########

FROM alpine:3.22 AS setup

ARG DOCKER_GUID \
	DOCKER_UID \
	DOCKER_TIME_CONT \
	DOCKER_TIME_CITY \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV VIRT=".build_packages"
ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

# This gets automatically updated via create_release.sh
ARG CURRENT_VERSION=v0.5.5

WORKDIR /app

RUN addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& apk --no-cache add --virtual ${VIRT} tzdata ca-certificates \
	&& cp /usr/share/zoneinfo/${TZ} /etc/localtime \
	&& update-ca-certificates \
	&& echo ${TZ} > /etc/timezone \
	&& apk del ${VIRT} \
	&& mkdir /db_data \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /db_data

	# Somewhat convoluted way to automatically select & download the correct package
RUN ARCH=$(uname -m) && \
    case "$ARCH" in \
        aarch64) SUFFIX=aarch64 ;; \
        armv6l) SUFFIX=armv6 ;; \
        *) exit 1 ;; \
    esac \
    && wget https://github.com/mrjackwills/belugasnooze_pi/releases/download/${CURRENT_VERSION}/belugasnooze_linux_${SUFFIX}.tar.gz \
    && tar xzvf belugasnooze_linux_${SUFFIX}.tar.gz belugasnooze \
    && rm belugasnooze_linux_${SUFFIX}.tar.gz \
    && chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /app/belugasnooze


##########
# RUNNER #
##########

FROM scratch

ARG DOCKER_TIME_CONT \
	DOCKER_TIME_CITY \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

COPY --from=setup /app/ /app
COPY --from=setup /etc/group /etc/passwd /etc/
COPY --from=setup /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

COPY --from=setup --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /db_data /db_data

USER ${DOCKER_APP_USER}

ENTRYPOINT ["/app/belugasnooze"]