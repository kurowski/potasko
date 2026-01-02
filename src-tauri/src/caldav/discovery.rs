use super::client::{CalDavClient, CalDavError, Result};
use super::types::CalendarInfo;
use super::xml;

/// Discover CalDAV endpoints and calendars.
///
/// Returns (principal_url, calendar_home_url, calendars).
pub async fn discover(client: &CalDavClient) -> Result<(String, String, Vec<CalendarInfo>)> {
    // Step 1: Find the user's principal URL
    let principal_url = discover_principal(client).await?;
    println!("Discovered principal: {}", principal_url);

    // Step 2: Find the calendar home URL
    let calendar_home_url = discover_calendar_home(client, &principal_url).await?;
    println!("Discovered calendar home: {}", calendar_home_url);

    // Step 3: List calendars with VTODO support
    let calendars = discover_calendars(client, &calendar_home_url).await?;
    println!("Discovered {} calendars", calendars.len());

    Ok((principal_url, calendar_home_url, calendars))
}

/// Discover the user's principal URL.
///
/// Tries well-known first, then falls back to PROPFIND on base URL.
async fn discover_principal(client: &CalDavClient) -> Result<String> {
    // Try well-known redirect first
    if let Ok(Some(location)) = client.get_well_known().await {
        // The redirect location might be the principal or we might need to extract it
        // Some servers redirect directly to principal, others to a base path
        // Try to get current-user-principal from the redirected location
        let principal = get_principal_from_propfind(client, &location).await;
        if let Ok(p) = principal {
            return Ok(p);
        }
        // If that fails, the location itself might be usable
        return Ok(location);
    }

    // Fall back to PROPFIND on base URL
    get_principal_from_propfind(client, "/").await
}

/// Get current-user-principal from a PROPFIND request.
async fn get_principal_from_propfind(client: &CalDavClient, path: &str) -> Result<String> {
    let response = client.propfind(path, 0, xml::propfind_principal()).await?;

    let responses = xml::parse_multistatus(&response)?;

    for resp in responses {
        if let Some(principal) = resp.props.current_user_principal {
            return Ok(principal);
        }
    }

    Err(CalDavError::Discovery(
        "Could not find current-user-principal".to_string(),
    ))
}

/// Discover the calendar home URL from the principal.
async fn discover_calendar_home(client: &CalDavClient, principal_url: &str) -> Result<String> {
    let response = client
        .propfind(principal_url, 0, xml::propfind_calendar_home())
        .await?;

    let responses = xml::parse_multistatus(&response)?;

    for resp in responses {
        if let Some(home) = resp.props.calendar_home_set {
            return Ok(home);
        }
    }

    Err(CalDavError::Discovery(
        "Could not find calendar-home-set".to_string(),
    ))
}

/// Discover calendars from the calendar home.
async fn discover_calendars(client: &CalDavClient, calendar_home: &str) -> Result<Vec<CalendarInfo>> {
    let response = client
        .propfind(calendar_home, 1, xml::propfind_calendars())
        .await?;

    let responses = xml::parse_multistatus(&response)?;

    let calendars: Vec<CalendarInfo> = responses
        .into_iter()
        .filter_map(xml::to_calendar_info)
        .collect();

    Ok(calendars)
}
