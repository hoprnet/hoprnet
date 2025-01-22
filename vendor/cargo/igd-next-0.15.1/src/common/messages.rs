use crate::PortMappingProtocol;
use std::net::SocketAddr;

// Content of the request.
pub const SEARCH_REQUEST: &str = "M-SEARCH * HTTP/1.1\r
Host:239.255.255.250:1900\r
ST:urn:schemas-upnp-org:device:InternetGatewayDevice:1\r
Man:\"ssdp:discover\"\r
MX:3\r\n\r\n";

pub const GET_EXTERNAL_IP_HEADER: &str = r#""urn:schemas-upnp-org:service:WANIPConnection:1#GetExternalIPAddress""#;

pub const ADD_ANY_PORT_MAPPING_HEADER: &str = r#""urn:schemas-upnp-org:service:WANIPConnection:1#AddAnyPortMapping""#;

pub const ADD_PORT_MAPPING_HEADER: &str = r#""urn:schemas-upnp-org:service:WANIPConnection:1#AddPortMapping""#;

pub const DELETE_PORT_MAPPING_HEADER: &str = r#""urn:schemas-upnp-org:service:WANIPConnection:1#DeletePortMapping""#;

pub const GET_GENERIC_PORT_MAPPING_ENTRY: &str =
    r#""urn:schemas-upnp-org:service:WANIPConnection:1#GetGenericPortMappingEntry""#;

const MESSAGE_HEAD: &str = r#"<?xml version="1.0"?>
<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
<s:Body>"#;

const MESSAGE_TAIL: &str = r#"</s:Body>
</s:Envelope>"#;

fn format_message(body: String) -> String {
    format!("{MESSAGE_HEAD}{body}{MESSAGE_TAIL}")
}

pub fn format_get_external_ip_message() -> String {
    r#"<?xml version="1.0"?>
<s:Envelope s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/" xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
    <s:Body>
        <m:GetExternalIPAddress xmlns:m="urn:schemas-upnp-org:service:WANIPConnection:1">
        </m:GetExternalIPAddress>
    </s:Body>
</s:Envelope>"#
    .into()
}

pub fn format_add_any_port_mapping_message(
    schema: &[String],
    protocol: PortMappingProtocol,
    external_port: u16,
    local_addr: SocketAddr,
    lease_duration: u32,
    description: &str,
) -> String {
    let args = schema
        .iter()
        .filter_map(|argument| {
            let value = match argument.as_str() {
                "NewEnabled" => 1.to_string(),
                "NewExternalPort" => external_port.to_string(),
                "NewInternalClient" => local_addr.ip().to_string(),
                "NewInternalPort" => local_addr.port().to_string(),
                "NewLeaseDuration" => lease_duration.to_string(),
                "NewPortMappingDescription" => description.to_string(),
                "NewProtocol" => protocol.to_string(),
                "NewRemoteHost" => "".to_string(),
                unknown => {
                    log::warn!("Unknown argument: {}", unknown);
                    return None;
                }
            };
            Some(format!("<{argument}>{value}</{argument}>"))
        })
        .collect::<Vec<_>>()
        .join("\n");

    format_message(format!(
        r#"<u:AddAnyPortMapping xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
        {args}
        </u:AddAnyPortMapping>"#,
    ))
}

pub fn format_add_port_mapping_message(
    schema: &[String],
    protocol: PortMappingProtocol,
    external_port: u16,
    local_addr: SocketAddr,
    lease_duration: u32,
    description: &str,
) -> String {
    let args = schema
        .iter()
        .filter_map(|argument| {
            let value = match argument.as_str() {
                "NewEnabled" => 1.to_string(),
                "NewExternalPort" => external_port.to_string(),
                "NewInternalClient" => local_addr.ip().to_string(),
                "NewInternalPort" => local_addr.port().to_string(),
                "NewLeaseDuration" => lease_duration.to_string(),
                "NewPortMappingDescription" => description.to_string(),
                "NewProtocol" => protocol.to_string(),
                "NewRemoteHost" => "".to_string(),
                unknown => {
                    log::warn!("Unknown argument: {}", unknown);
                    return None;
                }
            };
            Some(format!("<{argument}>{value}</{argument}>",))
        })
        .collect::<Vec<_>>()
        .join("\n");

    format_message(format!(
        r#"<u:AddPortMapping xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
        {args}
        </u:AddPortMapping>"#
    ))
}

pub fn format_delete_port_message(schema: &[String], protocol: PortMappingProtocol, external_port: u16) -> String {
    let args = schema
        .iter()
        .filter_map(|argument| {
            let value = match argument.as_str() {
                "NewExternalPort" => external_port.to_string(),
                "NewProtocol" => protocol.to_string(),
                "NewRemoteHost" => "".to_string(),
                unknown => {
                    log::warn!("Unknown argument: {}", unknown);
                    return None;
                }
            };
            Some(format!("<{argument}>{value}</{argument}>",))
        })
        .collect::<Vec<_>>()
        .join("\n");

    format_message(format!(
        r#"<u:DeletePortMapping xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
        {args}
        </u:DeletePortMapping>"#
    ))
}

pub fn formate_get_generic_port_mapping_entry_message(port_mapping_index: u32) -> String {
    format_message(format!(
        r#"<u:GetGenericPortMappingEntry xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
        <NewPortMappingIndex>{port_mapping_index}</NewPortMappingIndex>
        </u:GetGenericPortMappingEntry>"#
    ))
}
