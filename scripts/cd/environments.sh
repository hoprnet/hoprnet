# ===== Load env variables for the current github ref =====
# Takes:
# - GITHUB_REF
# - RELEASE
# Sets: 
# - RELEASE_NAME
# - RELEASE_IP
# - VERSION_MAJ_MIN
get_environment() {
  BRANCH=$(echo "$GITHUB_REF" | sed -e "s#refs/heads/##g") # Removing `refs/heads`

  if [ "$BRANCH" == 'master' ]; then
    RELEASE_NAME='master'
    RELEASE_IP='34.65.102.152'
    VERSION_MAJ_MIN='prerelease'
    return
  fi

  case "$BRANCH" in release/*)
    VERSION_MAJ_MIN=$(get_version_maj_min $RELEASE) 
    if [ "$VERSION_MAJ_MIN" == '1.58' ]; then
      RELEASE_NAME='queretaro'
      RELEASE_IP='34.65.207.39'
      return
    fi

    if [ "$VERSION_MAJ_MIN" == '1.57' ]; then
      RELEASE_NAME='larnaca'
      RELEASE_IP='unknown'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.56' ]; then
      RELEASE_NAME='luzern'
      RELEASE_IP='34.65.5.42'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.55' ]; then
      RELEASE_NAME='zug'
      RELEASE_IP='34.65.158.118'
      return
    fi
    if [ "$VERSION_MAJ_MIN" == '1.54' ]; then
      RELEASE_NAME='zurich'
      RELEASE_IP='unknown'
      return
    fi
    echo "Unknown version: $VERSION_MAJ_MIN"
  esac

  echo "Unknown release / environment: '$BRANCH'"
  #exit 1
  RELEASE_NAME='debug'
  RELEASE_IP='34.65.56.229'
}

