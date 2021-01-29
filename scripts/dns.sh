#!/bin/bash
set -e #u

# Get or create a TXT record within a release
# $1 = release name
# $2 = role (eg. bootstrap, node)
# $3 = multiaddress
gcloud_dns_txt_record() {
  # Multiple runs in the same machine will fail w/an existing transaction.yml file
  rm -f transaction.yaml
  local dns_entry="$1-$2.hoprnet.link"
  local txt_record=$(dig TXT +short $dns_entry)
  if [ -z "$txt_record" ]; then
    # echo "log | Dns Entry: $dns_entry"
    # Google takes some time to propagate their DNS entries, so it could be that the record has already been created.
    local maybe_txt_record=$(gcloud dns record-sets list --zone=hoprnet-link --name="$dns_entry."  --type="TXT")
    if [ -z "$maybe_txt_record" ]; then
      # echo "log | Status: Not created, creating"
      gcloud dns record-sets transaction start --zone="hoprnet-link"
      
      gcloud dns record-sets transaction add "dnsaddr=$3" \
        --name="$dns_entry" \
        --ttl="30" \
        --type="TXT" \
        --zone="hoprnet-link"

      # Piping out execute as it polutes stdout upon completion
      gcloud dns record-sets transaction execute --quiet --zone="hoprnet-link" 1>&2
      local txt_record=\""dnsaddr=$3"\"
    else
      # echo "log | Status: Created but not propagated."
      # Google echos “record-sets lists” in the form “NAME TYPE TTL DATA $dns_entry. TXT 30 "dnsaddr=..."'
      # echo "debug | Maybe Record: $maybe_txt_record"
      local txt_record=$(echo $maybe_txt_record | cut -f8 -d' ')
      # echo "debug | Record: $txt_record"
    fi
  fi
  echo $txt_record
}

# Get or create a TXT record within a release
# e.g. gcloud_txt_record master bootstrap '/ip4/34.65.204.200/tcp/9091/p2p/16Uiu2HAmNuRyLxM56eTEhNR8jPJrPYBbv9Foimz7EJAifL2BaeeF'
# $1 = release name
# $2 = role (eg. bootstrap, node)
# $3 = multiaddress
# Requires:
# - RELEASE_NAME
gcloud_txt_record() {
  # Workaround with file descriptors to avoid poluting stdout
  ( gcloud_dns_txt_record $1 $2 $3 3>&1 1>&2- 2>&3- ) | grep 'dnsaddr'
}
