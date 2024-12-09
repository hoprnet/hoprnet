flowchart RL
hoprnetwork((HOPR Network))
subgraph gclient[Gnosis VPN Client]
direction RL
service[Gnosis VPN Client] -.-> wgclient[Wireguard Client]
wgclient ==> hopr1udp
service --> hopr1http
subgraph hopr1[HOPR Node]
direction TB
hopr1http[HTTP API]
hopr1udp[UDP Socket]
hopr1p2p[P2P Connection]
hopr1udp ==> hopr1p2p
hopr1http --> hopr1p2p
end
end
subgraph gserver[Gnosis VPN Server]
direction LR
subgraph hopr2[HOPR Node]
direction TB
hopr2http[HTTP API]
hopr2udp[UDP Socket]
hopr2p2p[P2P Connection]
hopr2p2p ==> hopr2udp
hopr2p2p --> hopr2http
end
controller[Exit Controller] -.-> wgserver[Wireguard Server]
hopr2udp ==> wgserver
hopr2http --> controller
end
hopr1p2p --> hoprnetwork
hopr1p2p ==> hoprnetwork
hoprnetwork --> hopr2p2p
hoprnetwork ==> hopr2p2p
wgserver ==> destination[Destination Host]
client[Client] ==> wgclient
gnosischain[GnosisChain] -...-> service
gnosischain -.-> controller

style hopr1 fill:#ffffa0
style hopr2 fill:#ffffa0
style wgclient fill:#88171a,color:#fff
style wgserver fill:#88171a,color:#fff
style gclient fill:#d5cebc
style gserver fill:#d5cebc
style service fill:#3e6957,color:#fff
style controller fill:#3e6957,color:#fff
