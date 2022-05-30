# pi zero can't use > 3.12!
FROM node:16-alpine3.12

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=America \
	DOCKER_TIME_CITY=New_York \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV VIRT=".build_packages"
ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN deluser --remove-home node \
	&& addgroup -g ${DOCKER_GUID} -S ${DOCKER_APP_GROUP} \
	&& adduser -u ${DOCKER_UID} -S -G ${DOCKER_APP_GROUP} ${DOCKER_APP_USER} \
	&& apk --no-cache add --virtual ${VIRT} tzdata python3 make g++ 'su-exec=>0.2' \
	&& cp /usr/share/zoneinfo/${TZ} /etc/localtime \
	&& echo ${TZ} > /etc/timezone

RUN npm install -g npm@latest

WORKDIR /app

RUN mkdir /ip_address /logs \
	&& chown ${DOCKER_APP_USER}:${DOCKER_APP_GROUP} /app /ip_address /logs

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} package*.json tsconfig*.json .eslintignore .eslintrc.js ./

RUN su-exec ${DOCKER_APP_USER} npm install

RUN apk del ${VIRT}

USER ${DOCKER_APP_USER}

COPY --chown=${DOCKER_APP_USER}:${DOCKER_APP_GROUP} src /app/src

RUN npm run build \
	&& npm prune --production \
	&& npm cache clean --force

CMD [ "node", "dist/index.js"]