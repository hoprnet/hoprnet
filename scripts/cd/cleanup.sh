# Cleanup old gcloud resources.

if [ -z "$GCLOUD_INCLUDED" ]; then
  source scripts/cd/gcloud.sh 
fi

OLD_RELEASES='zurich zug luzern basodino'


cleanup_ips() {
  local IPS=$(gcloud compute addresses list)
  for x in $OLD_RELEASES
  do
    local IP=$(echo "$IPS" | grep $x | awk '{ print $2 }')
    if [ $IP ]; then
      local NAME=$(echo "$IPS" | grep $x | awk '{ print $1 }')
      echo "releasing - $NAME ($IP)"
      gcloud compute addresses delete $NAME $REGION --quiet
    fi
  done  
}

cleanup_ips
