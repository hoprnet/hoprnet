# Cleanup old gcloud resources.

if [ -z "${GCLOUD_INCLUDED:-}" ]; then
  source scripts/gcloud.sh
fi

if [ -z "${OLD_RELEASES:-}" ]; then
  source scripts/environments.sh
fi

# $1=IP
cleanup_ip() {
  local IP="${1}"

    if [ $IP ]; then
      local NAME=$(echo "$IPS" | grep "${release_name}" | awk '{ print $1 }')
      echo "- releasing - $NAME ($IP)"
      gcloud compute addresses delete $NAME $REGION --quiet
    fi
  
}

cleanup_ips() {
  local IPS=$(gcloud compute addresses list)
  for release_name in $OLD_RELEASES
  do
    local IP=$(echo "$IPS" | grep -q "${release_name}" | awk '{ print $2 }')    
    cleanup_ip "${release_name}"
  done
}

# $1=release name
cleanup_instance() {
  local release_name="${1}"
  
  while read -r instance; do
    local name=$(echo "$instance" | awk '{ print $1 }')
    echo "- stopping $name"
    gcloud compute instances stop $name $ZONE
  done < <(gcloud compute instances list --filter="name~'^${release_name}-' AND status=RUNNING" | tail -n +2)

  while read -r instance; do
    local name=$(echo "$instance" | awk '{ print $1 }')
    local zone=$(echo "$instance" | awk '{ print $2 }')
    echo "- deleting terminated instance $name"
    gcloud compute instances delete $name --zone=$zone --keep-disks=data --quiet
  done < <(gcloud compute instances list --filter="name~'^${release_name}-' AND status=TERMINATED" | tail -n +2)
}

cleanup_instances() {
  for release_name in $OLD_RELEASES
  do
    cleanup_instance "${release_name}"  
  done
}

cleanup () {
  cleanup_ips
  cleanup_instances
}

if [ "$0" = "./scripts/cleanup.sh" ]; then
  cleanup
fi
