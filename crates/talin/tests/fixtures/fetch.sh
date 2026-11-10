#!/bin/sh
# Downloads the media the multimodal tests attach. The files themselves are not tracked.
set -eu
cd "$(dirname "$0")"

wget -O stop_sign.jpg \
    "https://upload.wikimedia.org/wikipedia/commons/thumb/f/f9/STOP_sign.jpg/250px-STOP_sign.jpg"
wget -O dummy.pdf \
    "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf"
