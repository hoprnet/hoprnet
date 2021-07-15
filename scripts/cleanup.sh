# Cleanup old gcloud resources.

if [ -z "${GCLOUD_INCLUDED:-}" ]; then
  source scripts/gcloud.sh
fi

if [ -z "${OLD_RELEASES:-}" ]; then
  source scripts/environments.sh
fi


cleanup_ips() {
  local IPS=$(gcloud compute addresses list)
  for x in $OLD_RELEASES
  do
    local IP=$(echo "$IPS" | grep -q $x | awk '{ print $2 }')
    if [ $IP ]; then
      local NAME=$(echo "$IPS" | grep $x | awk '{ print $1 }')
      echo "- releasing - $NAME ($IP)"
      gcloud compute addresses delete $NAME $REGION --quiet
    fi
  done
}

cleanup_instances() {
  local instances
  for old in $OLD_RELEASES
  do
    while read -r instance; do
      local name=$(echo "$instance" | awk '{ print $1 }')
      echo "- stopping $name"
      gcloud compute instances stop $name $ZONE
    done < <(gcloud compute instances list --filter="name~'^${old}-' AND status=RUNNING" | tail -n +2)

    while read -r instance; do
      local name=$(echo "$instance" | awk '{ print $1 }')
      local zone=$(echo "$instance" | awk '{ print $2 }')
      echo "- deleting terminated instance $name"
      gcloud compute instances delete $name --zone=$zone --keep-disks=data --quiet
    done < <(gcloud compute instances list --filter="name~'^${old}-' AND status=TERMINATED" | tail -n +2)
  done
}

cleanup () {
  cleanup_ips
  cleanup_instances
}

if [ "$0" = "./scripts/cleanup.sh" ]; then
  cleanup
fi
