#!/usr/bin/env bash
set -euo pipefail
: "${CANOPY_RELAY_SERVICE:?set service name}"
: "${CANOPY_RELAY_REGION:?set region}"
: "${CANOPY_RELAY_REVISION:?set verified rollback revision}"
gcloud run services update-traffic "$CANOPY_RELAY_SERVICE" \
  --region "$CANOPY_RELAY_REGION" --to-revisions "$CANOPY_RELAY_REVISION=100"
