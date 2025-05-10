# generate smart contract bindings
generate-bindings:
    cd ethereum/contracts; \
    forge bind --offline --bindings-path ./../bindings/src/codegen \
      --module --skip-extra-derives --alloy --overwrite \
      --select '^(HoprAnnouncements|HoprAnnouncementsEvents|HoprCapabilityPermissions|HoprChannels|HoprChannelsEvents|HoprCrypto|HoprDummyProxyForNetworkRegistry|HoprBoost|HoprToken|HoprLedger|HoprLedgerevents|HoprMultisig|HoprNetworkRegistry|HoprNetworkRegistryEvents|HoprNodeManagementModule|HoprNodeSafeRegistry|HoprNodeSafeRegistryEvents|HoprNodeStakeFactory|HoprNodeStakeFactoryEvents|HoprSafeProxyForNetworkRegistry|HoprStakingProxyForNetworkRegistry|HoprTicketPriceOracle|HoprTicketPriceOracleEvents|HoprWinningProbabilityOracle|HoprWinningProbabilityOracleEvents)$'

