//! Bindings for [`SCNetworkConfiguration`].
//!
//! [`SCNetworkConfiguration`]: https://developer.apple.com/documentation/systemconfiguration/scnetworkconfiguration?language=objc
use core_foundation::{array::CFArray, base::TCFType, string::CFString};
use system_configuration_sys::network_configuration::{
    SCNetworkInterfaceCopyAll, SCNetworkInterfaceGetBSDName, SCNetworkInterfaceGetInterfaceType,
    SCNetworkInterfaceGetLocalizedDisplayName, SCNetworkInterfaceGetTypeID, SCNetworkInterfaceRef,
};

core_foundation::declare_TCFType!(
    /// Represents a network interface.
    ///
    /// See [`SCNetworkInterfaceRef`] and its [methods] for details.
    ///
    /// [`SCNetworkInterfaceRef`]: https://developer.apple.com/documentation/systemconfiguration/scnetworkinterfaceref?language=objc
    /// [methods]: https://developer.apple.com/documentation/systemconfiguration/scnetworkconfiguration?language=objc
    SCNetworkInterface,
    SCNetworkInterfaceRef
);
core_foundation::impl_TCFType!(
    SCNetworkInterface,
    SCNetworkInterfaceRef,
    SCNetworkInterfaceGetTypeID
);

// TODO: implement all the other methods a SCNetworkInterface has
impl SCNetworkInterface {
    /// Get type of the network interface, if the type is recognized, returns `None` otherwise.
    ///
    /// See [`SCNetworkInterfaceGetInterfaceType`] for details.
    ///
    /// [`SCNetworkInterfaceGetInterfaceType`]: https://developer.apple.com/documentation/systemconfiguration/1517371-scnetworkinterfacegetinterfacety?language=objc
    pub fn interface_type(&self) -> Option<SCNetworkInterfaceType> {
        SCNetworkInterfaceType::from_cfstring(&self.interface_type_string()?)
    }

    /// Returns the raw interface type identifier.
    ///
    /// See [`SCNetworkInterfaceGetInterfaceType`] for details.
    ///
    /// [`SCNetworkInterfaceGetInterfaceType`]: https://developer.apple.com/documentation/systemconfiguration/1517371-scnetworkinterfacegetinterfacety?language=objc
    pub fn interface_type_string(&self) -> Option<CFString> {
        unsafe {
            let ptr = SCNetworkInterfaceGetInterfaceType(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(CFString::wrap_under_get_rule(ptr))
            }
        }
    }

    /// Returns the _BSD_ name for the interface, such as `en0`.
    ///
    /// See [`SCNetworkInterfaceGetBSDName`] for details.
    ///
    /// [`SCNetworkInterfaceGetBSDName`]: https://developer.apple.com/documentation/systemconfiguration/1516854-scnetworkinterfacegetbsdname?language=objc
    pub fn bsd_name(&self) -> Option<CFString> {
        unsafe {
            let ptr = SCNetworkInterfaceGetBSDName(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(CFString::wrap_under_get_rule(ptr))
            }
        }
    }

    /// Returns the localized display name for the interface.
    ///
    /// See [`SCNetworkInterfaceGetLocalizedDisplayName`] for details.
    ///
    /// [`SCNetworkInterfaceGetLocalizedDisplayName`]: https://developer.apple.com/documentation/systemconfiguration/1517060-scnetworkinterfacegetlocalizeddi?language=objc
    pub fn display_name(&self) -> Option<CFString> {
        unsafe {
            let ptr = SCNetworkInterfaceGetLocalizedDisplayName(self.0);
            if ptr.is_null() {
                None
            } else {
                Some(CFString::wrap_under_get_rule(ptr))
            }
        }
    }
}

/// Represents the possible network interface types.
///
/// See [_Network Interface Types_] documentation for details.
///
/// [_Network Interface Types_]: https://developer.apple.com/documentation/systemconfiguration/scnetworkconfiguration/network_interface_types?language=objc
#[derive(Debug)]
pub enum SCNetworkInterfaceType {
    /// A 6to4 interface.
    SixToFour,
    /// Bluetooth interface.
    Bluetooth,
    /// Bridge interface.
    Bridge,
    /// Ethernet bond interface.
    Bond,
    /// Ethernet interface.
    Ethernet,
    /// FireWire interface.
    FireWire,
    /// IEEE80211 interface.
    IEEE80211,
    /// IPSec interface.
    IPSec,
    /// IrDA interface.
    IrDA,
    /// L2TP interface.
    L2TP,
    /// Modem interface.
    Modem,
    /// PPP interface.
    PPP,
    /// PPTP interface.
    ///
    /// Deprecated, one should use the PPP variant.
    PPTP,
    /// Serial interface.
    Serial,
    /// VLAN interface.
    VLAN,
    /// WWAN interface.
    WWAN,
    /// IPv4 interface.
    IPv4,
}

impl SCNetworkInterfaceType {
    /// Tries to construct a type by matching it to string constants used to identify a network
    /// interface type. If no constants match it, `None` is returned.
    pub fn from_cfstring(type_id: &CFString) -> Option<Self> {
        use system_configuration_sys::network_configuration::*;

        let id_is_equal_to = |const_str| -> bool {
            let const_str = unsafe { CFString::wrap_under_get_rule(const_str) };
            &const_str == type_id
        };
        unsafe {
            if id_is_equal_to(kSCNetworkInterfaceType6to4) {
                Some(SCNetworkInterfaceType::SixToFour)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeBluetooth) {
                Some(SCNetworkInterfaceType::Bluetooth)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeBridge) {
                Some(SCNetworkInterfaceType::Bridge)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeBond) {
                Some(SCNetworkInterfaceType::Bond)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeEthernet) {
                Some(SCNetworkInterfaceType::Ethernet)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeFireWire) {
                Some(SCNetworkInterfaceType::FireWire)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeIEEE80211) {
                Some(SCNetworkInterfaceType::IEEE80211)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeIPSec) {
                Some(SCNetworkInterfaceType::IPSec)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeIrDA) {
                Some(SCNetworkInterfaceType::IrDA)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeL2TP) {
                Some(SCNetworkInterfaceType::L2TP)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeModem) {
                Some(SCNetworkInterfaceType::Modem)
            } else if id_is_equal_to(kSCNetworkInterfaceTypePPP) {
                Some(SCNetworkInterfaceType::PPP)
            } else if id_is_equal_to(kSCNetworkInterfaceTypePPTP) {
                Some(SCNetworkInterfaceType::PPTP)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeSerial) {
                Some(SCNetworkInterfaceType::Serial)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeVLAN) {
                Some(SCNetworkInterfaceType::VLAN)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeWWAN) {
                Some(SCNetworkInterfaceType::WWAN)
            } else if id_is_equal_to(kSCNetworkInterfaceTypeIPv4) {
                Some(SCNetworkInterfaceType::IPv4)
            } else {
                None
            }
        }
    }
}

/// Retrieve all current network interfaces
///
/// See [`SCNetworkInterfaceCopyAll`] for more details.
///
/// [`SCNetworkInterfaceCopyAll`]: https://developer.apple.com/documentation/systemconfiguration/1517090-scnetworkinterfacecopyall?language=objc
pub fn get_interfaces() -> CFArray<SCNetworkInterface> {
    unsafe { CFArray::<SCNetworkInterface>::wrap_under_create_rule(SCNetworkInterfaceCopyAll()) }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_all_interfaces() {
        let _ = get_interfaces();
    }

    #[test]
    fn test_get_type() {
        for iface in get_interfaces().into_iter() {
            if iface.interface_type().is_none() {
                panic!(
                    "Interface  {:?} ({:?}) has unrecognized type {:?}",
                    iface.display_name(),
                    iface.bsd_name(),
                    iface.interface_type_string()
                )
            }
        }
    }
}
