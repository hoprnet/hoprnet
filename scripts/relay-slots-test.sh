# prevent souring of this script, only allow execution
$(return >/dev/null 2>&1)
test "$?" -eq "0" && { echo "This script should only be executed." >&2; exit 1; }
          
# set log id and use shared log function for readable logs
declare mydir
mydir=$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd -P)
declare HOPR_LOG_ID="hopr-connect-test"
source "${mydir}/utils.sh"

# exit on errors, undefined variables, ensure errors in pipes are not hidden
set -Eeuo pipefail

source "${mydir}/common.sh"

setup "relay-slots"

# run alice (client)
# should be able to send 'test from alice' to bob through relay charly
# should be ablt to get 'echo: test' back from bob
start_node tests/node.ts \
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
      },
      {
        'cmd': 'wait',
        'waitForSecs': 2
      },
      { 
        'cmd': 'hangup',
        'targetIdentityName': 'bob'
      }
    ]" \
    --port ${alice_port} \
    --pipeFile "${alice_pipe}" \
    --identityName 'alice' \
    --bootstrapPort ${charly_port} \
    --bootstrapIdentityName 'charly' \
    --noDirectConnections true \
    --noWebRTCUpgrade false \
    
# run bob (client)
# should be able to receive 'test' from alice through charly
# should be able to reply with 'echo: test'
start_node tests/node.ts "${bob_log}"  \
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
  --noWebRTCUpgrade false \  
  
# run charly
# should able to serve as a bootstrap
# should be able to relay 1 connection at a time
start_node tests/node.ts "${charly_log}" \
  "[]" \
  --port ${charly_port} \
  --identityName 'charly' \
  --noDirectConnections true \
  --noWebRTCUpgrade false \
  --maxRelayedConnections 1 \
  --relayFreeTimeout 2000 # to simulate relay being busy

# run dave (client)
# should try connecting to bob through relay charly and get RELAY_FULL error
start_node tests/node.ts "${dave_log}" \
  "[ {
        'cmd': 'wait',
        'waitForSecs': 3
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
        'msg': 'test from dave'
      }
    ]" \
  --port ${dave_port} \
  --identityName 'dave' \
  --bootstrapPort ${charly_port} \
  --bootstrapIdentityName 'charly' \
  --noDirectConnections true \
  --noWebRTCUpgrade false

# run ed (client)
# should try connecting to bob through relay charly after alice finishes talking to bob and succeed
start_node tests/node.ts "${ed_log}" \
  "[ {
        'cmd': 'wait',
        'waitForSecs': 6
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
        'msg': 'test from ed'
      }
    ]" \
  --port ${ed_port} \
  --identityName 'ed' \
  --bootstrapPort ${charly_port} \
  --bootstrapIdentityName 'charly' \
  --noDirectConnections true \
  --noWebRTCUpgrade false

# wait till nodes finish communicating
wait_for_regex_in_file "${alice_log}" "all tasks executed"
wait_for_regex_in_file "${bob_log}" "all tasks executed"
wait_for_regex_in_file "${charly_log}" "all tasks executed"
wait_for_regex_in_file "${ed_log}" "all tasks executed"

# dave should have failed to complete
wait_for_regex_in_file "${dave_log}" "Answer was: <FAIL_RELAY_FULL>"
wait_for_regex_in_file "${dave_log}" "dialProtocol to bob failed"

# create global flow log
rm -Rf "${flow_log}"
cat "${alice_log}" | sed -En 's/hopr-connect.*FLOW: /alice: /p' >> "${flow_log}"
cat "${bob_log}" | sed -En 's/hopr-connect.*FLOW: /bob: /p' >> "${flow_log}"
cat "${charly_log}" | sed -En 's/hopr-connect.*FLOW: /charly: /p' >> "${flow_log}"
cat "${dave_log}" | sed -En 's/hopr-connect.*FLOW: /dave: /p' >> "${flow_log}"
sort -k1,1 --stable --output "${flow_log}" "${flow_log}"

expect_file_content "${alice_pipe}" \
">bob: test from alice
<bob: echo: test from alice"

expect_file_content "${bob_pipe}" \
"<alice: test from alice
>alice: echo: test from alice
<ed: test from ed
>ed: echo: test from ed"

teardown