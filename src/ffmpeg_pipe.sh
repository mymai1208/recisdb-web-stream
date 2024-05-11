#!/bin/bash

DEVICE=${1}
CHANNEL=${2}

recisdb tune --device ${DEVICE} --no-strip --channel ${CHANNEL} - | ffmpeg -i pipe:0 -vcodec libx264 -acodec copy -f mpegts pipe:1