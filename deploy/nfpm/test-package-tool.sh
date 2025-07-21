#!/bin/bash
set -Eeuo pipefail
#set -x

PROJECT_ID="${GCP_PROJECT:-hoprassociation}"
ZONE="europe-west4-a"
MACHINE_TYPE_x86="e2-medium"      # GCP's x86_64 instances
MACHINE_TYPE_ARM="t2a-standard-2" # GCP's ARM instances
NETWORK="default"
SUBNET="default"

get_vm_image() {
  case "${DISTRIBUTION}" in
  deb)
    if [ "${ARCHITECTURE}" == "aarch64-linux" ]; then
      echo "projects/debian-cloud/global/images/family/debian-12-arm64"
      return
    else
      echo "projects/debian-cloud/global/images/family/debian-12"
      return
    fi
    ;;
  rpm)
    if [ "${ARCHITECTURE}" == "aarch64-linux" ]; then
      echo "projects/centos-cloud/global/images/family/centos-stream-9-arm64"
      return
    else
      echo "projects/centos-cloud/global/images/family/centos-stream-9"
      return
    fi
    ;;
  archlinux)
    # https://github.com/GoogleCloudPlatform/compute-archlinux-image-builder
    echo "projects/arch-linux-gce/global/images/family/arch"
    ;;
  *)
    echo "Unsupported distribution: ${DISTRIBUTION}. Supported distributions are: deb, rpm, archlinux."
    exit 1
    ;;
  esac
}

create_action() {
  local image
  image=$(get_vm_image "${DISTRIBUTION}")

  echo "Creating VM for distribution: $DISTRIBUTION, architecture: $ARCHITECTURE"
  if [ "${ARCHITECTURE}" == "aarch64-linux" ]; then
    machine_type="$MACHINE_TYPE_ARM"
  else
    machine_type="$MACHINE_TYPE_x86"
  fi
  image_project=$(echo "$image" | cut -d'/' -f2)
  gcloud compute instances create "${INSTANCE_NAME}" \
    --project=${PROJECT_ID} \
    --zone=${ZONE} \
    --machine-type=${machine_type} \
    --network=${NETWORK} \
    --subnet=${SUBNET} \
    --image-project="${image_project}" \
    --image-family="${image##*/}" \
    --boot-disk-size=200GB \
    --tags="iap,hoprd-p2p" \
    --scopes=storage-ro \
    --create-disk=auto-delete=yes \
    --quiet \
    --metadata=startup-script='#! /bin/bash
      mkdir -p /etc/hoprd
      sudo chmod 777 /etc/hoprd
    '
  sleep 15
  waiting_iterations=0
  # Automatically delete the VM after 1 hour
  while ! gcloud compute ssh --tunnel-through-iap --project=${PROJECT_ID} --zone=${ZONE} "${INSTANCE_NAME}" --command="sudo mkdir -p /etc/hoprd && sudo chmod 777 /etc/hoprd && echo SSH is accessible" --quiet 2>/dev/null; do
    echo "Waiting for SSH to become accessible..."
    waiting_iterations=$((waiting_iterations + 1))
    if [ $waiting_iterations -ge 33 ]; then
      echo "SSH is still not accessible after 3 minutes. Exiting."
      echo "You can try to SSH into the VM using the following command:"
      echo "gcloud compute ssh --tunnel-through-iap --project=${PROJECT_ID} --zone=${ZONE} ${INSTANCE_NAME}"
      exit 1
    fi
    sleep 5
  done
  echo "SSH is now accessible on ${INSTANCE_NAME}."
  echo "VM ${INSTANCE_NAME} created successfully. You can SSH into the VM using the following command:"
  echo "gcloud compute ssh --tunnel-through-iap --project=${PROJECT_ID} --zone=${ZONE} ${INSTANCE_NAME}"
}

copy_action() {
  echo "Copying artifacts on ${INSTANCE_NAME}"
  script_dir=$(cd "$(dirname "$0")" && pwd)
  if [ -f "${script_dir}/hopr.id" ]; then
    gcloud compute scp --tunnel-through-iap --project=${PROJECT_ID} --zone=${ZONE} "${script_dir}/hopr.id" "${INSTANCE_NAME}":/etc/hoprd/hopr.id
    gcloud compute ssh --tunnel-through-iap --project=${PROJECT_ID} --zone=${ZONE} "${INSTANCE_NAME}" --command="sudo chown root:root /etc/hoprd/hopr.id && sudo chmod 644 /etc/hoprd/hopr.id" --quiet 2>/dev/null
  fi
  gcloud compute scp --tunnel-through-iap --project=${PROJECT_ID} --zone=${ZONE} "${script_dir}/install-hoprd-package.sh" "${INSTANCE_NAME}":/tmp/install-hoprd-package.sh
  gcloud compute scp --tunnel-through-iap --project=${PROJECT_ID} --zone=${ZONE} "${script_dir}/../../dist/packages/hoprd-${ARCHITECTURE}.${DISTRIBUTION}" "${INSTANCE_NAME}":/tmp/hoprd."${DISTRIBUTION}"
  echo "Artifacts successfully copied on ${INSTANCE_NAME}"
}

install_action() {
  echo "Installing hoprd package on ${INSTANCE_NAME}"
  gcloud compute ssh --tunnel-through-iap --project=${PROJECT_ID} --zone=${ZONE} "${INSTANCE_NAME}" --command="sudo bash /tmp/install-hoprd-package.sh ${DISTRIBUTION} ${HOPRD_PASSWORD} ${HOPRD_SAFE_ADDRESS} ${HOPRD_MODULE_ADDRESS} ${HOPRD_PROVIDER}"
  echo "Package installed successfully on ${DISTRIBUTION}-${ARCHITECTURE}."
}

delete_action() {
  echo "Deleting VM for distribution: $DISTRIBUTION, architecture: $ARCHITECTURE"
  if ! gcloud compute instances delete "${INSTANCE_NAME}" --project=${PROJECT_ID} --zone=${ZONE} --quiet; then
    echo "Failed to delete VM ${INSTANCE_NAME}. It may not exist or there was an error."
    exit 1
  fi
  echo "VM hoprd-node-${DISTRIBUTION}-${ARCHITECTURE} deleted successfully."
}

check_parameters() {
  if [ $# -ne 3 ]; then
    echo "Usage: $0 <action> <distribution> <architecture>"
    echo "Example: $0 create deb x86_64-linux "
    exit 1
  fi

  if [ "$ARCHITECTURE" != "x86_64-linux" ] && [ "$ARCHITECTURE" != "aarch64-linux" ]; then
    echo "Unsupported architecture: $ARCHITECTURE. Supported architectures are x86_64-linux and aarch64-linux"
    exit 1
  fi

  if [ "$DISTRIBUTION" != "deb" ] && [ "$DISTRIBUTION" != "rpm" ] && [ "$DISTRIBUTION" != "archlinux" ]; then
    echo "Unsupported distribution: $DISTRIBUTION. Supported distributions are deb, rpm, and archlinux."
    exit 1
  fi
}

main() {
  case "$ACTION" in
  create)
    create_action
    ;;
  copy)
    copy_action
    ;;
  install)
    install_action
    ;;
  delete)
    delete_action
    ;;
  *)
    echo "Invalid action specified. Use 'create' or 'delete'."
    exit 1
    ;;
  esac
}

ACTION="$1"       # e.g., "create", "copy", "install", "delete"
DISTRIBUTION="$2" # e.g., "deb", "rpm", "archlinux"
ARCHITECTURE="$3" # e.g., "x86_64-linux", "aarch64-linux"
INSTANCE_NAME="hoprd-node-${DISTRIBUTION}-${ARCHITECTURE/_/-}"
INSTANCE_NAME="${INSTANCE_NAME/-linux/}" # Remove -linux suffix for VM name
check_parameters "$@"
main "$@"
