# prevent sourcing of this script, only allow execution
$(return > /dev/null 2>&1)
test "$?" -eq "0" && {
  echo "This script should only be executed." >&2
  exit 1
}

# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &> /dev/null && pwd -P)
declare HOPR_LOG_ID="hopr-connect-test"
source "${mydir}/../utils.sh"

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

source "${mydir}/common.sh"

setup "reconnect"

# run alice (client)
# should be able to send 'test from alice' to bob through relay charly
start_node tests/node \
  "${alice_log}" \
  "[ 
      {
        'cmd': 'wait',
        'waitForSecs': 2
      },
      {
        'cmd': 'dial',
        'targetIdentityName': 'charly',
        'targetPort': ${charly_port}
      },
      {
        'cmd': 'msg',
        'relayIdentityName': 'charly',
        'targetIdentityName': 'bob',
        'msg': 'test from alice'
      }     
    ]" \
  --port ${alice_port} \
  --pipeFile "${alice_pipe}" \
  --identityName 'alice' \
  --bootstrapPort ${charly_port} \
  --bootstrapIdentityName 'charly' \
  --noDirectConnections true \
  --noWebRTCUpgrade true \
  --useLocalAddress true

# run bob (client)
# should be able to receive 'test' from alice through charly
# should be able to reply with 'echo: test'
start_node tests/node "${bob_log}" \
  "[ {
        'cmd': 'wait',
        'waitForSecs': 2
      },
      {
        'cmd': 'dial',
        'targetIdentityName': 'charly',
        'targetPort': ${charly_port}
      }      
    ]" \
  --port ${bob_port} \
  --pipeFile "${bob_pipe}" \
  --identityName 'bob' \
  --bootstrapPort ${charly_port} \
  --bootstrapIdentityName 'charly' \
  --noDirectConnections true \
  --noWebRTCUpgrade true \
  --useLocalAddress true

# run charly
# should able to serve as a bootstrap
# should be able to relay 1 connection at a time
start_node tests/node "${charly_log}" \
  "[]" \
  --port ${charly_port} \
  --identityName 'charly' \
  --noDirectConnections true \
  --noWebRTCUpgrade false \
  --useLocalAddress true

# wait till nodes finish communicating
wait_for_regex "${alice_log}" "all tasks executed"
wait_for_regex "${bob_log}" "all tasks executed"

# wait a little
sleep 1

# run another instance of alice
start_node tests/node \
  "${alice2_log}" \
  "[ 
      {
        'cmd': 'wait',
        'waitForSecs': 2
      },
      {
        'cmd': 'dial',
        'targetIdentityName': 'charly',
        'targetPort': ${charly_port}
      },
      {
        'cmd': 'msg',
        'relayIdentityName': 'charly',
        'targetIdentityName': 'bob',
        'msg': 'test2 from alice'
      }
    ]" \
  --port ${alice2_port} \
  --pipeFile "${alice2_pipe}" \
  --identityName 'alice' \
  --bootstrapPort ${charly_port} \
  --bootstrapIdentityName 'charly' \
  --noDirectConnections true \
  --noWebRTCUpgrade true \
  --useLocalAddress true

# wait for the second alice to finish sending
wait_for_regex "${alice2_log}" "all tasks executed"

# bob should have received RESTART status msg
wait_for_regex "${bob_log}" "RESTART received. Ending stream"

# bob should have received both messages from alice1 and alice2
expect_file_content "${bob_pipe}" \
  "<alice: test from alice
>alice: echo: test from alice
<alice: test2 from alice
>alice: echo: test2 from alice"

teardown
