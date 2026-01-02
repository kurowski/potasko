use roxmltree::{Document, Node};

use super::client::CalDavError;
use super::types::CalendarInfo;

// DAV namespace
const DAV_NS: &str = "DAV:";
// CalDAV namespace
const CALDAV_NS: &str = "urn:ietf:params:xml:ns:caldav";
// Apple calendar server namespace (for color)
const APPLE_NS: &str = "http://apple.com/ns/ical/";
// Calendar server namespace (for ctag)
const CS_NS: &str = "http://calendarserver.org/ns/";

/// Parse a DAV:multistatus response and extract href + properties for each resource.
pub fn parse_multistatus(xml: &str) -> Result<Vec<DavResponse>, CalDavError> {
    let doc = Document::parse(xml).map_err(|e| CalDavError::XmlParse(e.to_string()))?;

    let mut responses = Vec::new();

    for response_node in doc.descendants().filter(|n| n.has_tag_name((DAV_NS, "response"))) {
        if let Some(response) = parse_response(&response_node) {
            responses.push(response);
        }
    }

    Ok(responses)
}

/// A single response from a multistatus.
#[derive(Debug)]
pub struct DavResponse {
    pub href: String,
    pub props: DavProps,
}

/// Extracted DAV properties.
#[derive(Debug, Default)]
pub struct DavProps {
    pub current_user_principal: Option<String>,
    pub calendar_home_set: Option<String>,
    pub display_name: Option<String>,
    pub resource_type: ResourceType,
    pub supported_components: Vec<String>,
    pub calendar_color: Option<String>,
    pub ctag: Option<String>,
}

#[derive(Debug, Default, PartialEq)]
pub struct ResourceType {
    pub is_collection: bool,
    pub is_calendar: bool,
}

fn parse_response(node: &Node) -> Option<DavResponse> {
    let href = node
        .descendants()
        .find(|n| n.has_tag_name((DAV_NS, "href")))?
        .text()?
        .to_string();

    let mut props = DavProps::default();

    // Look for propstat with successful status
    for propstat in node.descendants().filter(|n| n.has_tag_name((DAV_NS, "propstat"))) {
        // Check if this propstat has a successful status
        let status = propstat
            .descendants()
            .find(|n| n.has_tag_name((DAV_NS, "status")))
            .and_then(|n| n.text())
            .unwrap_or("");

        if !status.contains("200") {
            continue;
        }

        // Extract properties from the prop element
        if let Some(prop) = propstat.descendants().find(|n| n.has_tag_name((DAV_NS, "prop"))) {
            parse_props(&prop, &mut props);
        }
    }

    Some(DavResponse { href, props })
}

fn parse_props(prop: &Node, props: &mut DavProps) {
    for child in prop.children().filter(|n| n.is_element()) {
        match (child.tag_name().namespace(), child.tag_name().name()) {
            // DAV:current-user-principal
            (Some(DAV_NS), "current-user-principal") => {
                props.current_user_principal = child
                    .descendants()
                    .find(|n| n.has_tag_name((DAV_NS, "href")))
                    .and_then(|n| n.text())
                    .map(|s| s.to_string());
            }

            // CalDAV:calendar-home-set
            (Some(CALDAV_NS), "calendar-home-set") => {
                props.calendar_home_set = child
                    .descendants()
                    .find(|n| n.has_tag_name((DAV_NS, "href")))
                    .and_then(|n| n.text())
                    .map(|s| s.to_string());
            }

            // DAV:displayname
            (Some(DAV_NS), "displayname") => {
                props.display_name = child.text().map(|s| s.to_string());
            }

            // DAV:resourcetype
            (Some(DAV_NS), "resourcetype") => {
                for rt_child in child.children().filter(|n| n.is_element()) {
                    match (rt_child.tag_name().namespace(), rt_child.tag_name().name()) {
                        (Some(DAV_NS), "collection") => props.resource_type.is_collection = true,
                        (Some(CALDAV_NS), "calendar") => props.resource_type.is_calendar = true,
                        _ => {}
                    }
                }
            }

            // CalDAV:supported-calendar-component-set
            (Some(CALDAV_NS), "supported-calendar-component-set") => {
                for comp in child.descendants().filter(|n| n.has_tag_name((CALDAV_NS, "comp"))) {
                    if let Some(name) = comp.attribute("name") {
                        props.supported_components.push(name.to_string());
                    }
                }
            }

            // Apple:calendar-color
            (Some(APPLE_NS), "calendar-color") => {
                props.calendar_color = child.text().map(|s| s.to_string());
            }

            // CS:getctag
            (Some(CS_NS), "getctag") => {
                props.ctag = child.text().map(|s| s.to_string());
            }

            _ => {}
        }
    }
}

/// Convert a DavResponse to CalendarInfo if it represents a calendar with VTODO support.
pub fn to_calendar_info(response: DavResponse) -> Option<CalendarInfo> {
    let props = response.props;

    // Must be a calendar collection
    if !props.resource_type.is_calendar {
        return None;
    }

    // Check if VTODO is supported (if supported_components is empty, assume all are supported)
    let supports_vtodo = props.supported_components.is_empty()
        || props.supported_components.iter().any(|c| c == "VTODO");

    Some(CalendarInfo {
        href: response.href,
        display_name: props.display_name,
        color: props.calendar_color,
        ctag: props.ctag,
        supports_vtodo,
    })
}

// XML request templates

/// PROPFIND to get current-user-principal.
pub fn propfind_principal() -> &'static str {
    r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:">
  <d:prop>
    <d:current-user-principal/>
  </d:prop>
</d:propfind>"#
}

/// PROPFIND to get calendar-home-set.
pub fn propfind_calendar_home() -> &'static str {
    r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <c:calendar-home-set/>
  </d:prop>
</d:propfind>"#
}

/// PROPFIND to list calendars with their properties.
pub fn propfind_calendars() -> &'static str {
    r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav" xmlns:a="http://apple.com/ns/ical/" xmlns:cs="http://calendarserver.org/ns/">
  <d:prop>
    <d:displayname/>
    <d:resourcetype/>
    <c:supported-calendar-component-set/>
    <a:calendar-color/>
    <cs:getctag/>
  </d:prop>
</d:propfind>"#
}
