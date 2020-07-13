FROM debian:10-slim

COPY jobs.crontab jobs.crontab

RUN apt-get update && apt-get install -y \
  cron mecab libmecab-dev

RUN crontab jobs.crontab
ENTRYPOINT ["cron", "-f", "-L", "15"]