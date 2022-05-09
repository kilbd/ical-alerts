use aws_lambda_events::{
    apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse},
    encodings::Body,
};
use http::HeaderMap;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use simple_logger::SimpleLogger;

#[tokio::main]
async fn main() -> Result<(), Error> {
    SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .unwrap();
    let handler_fn = service_fn(handler);
    lambda_runtime::run(handler_fn).await?;
    Ok(())
}

async fn handler(
    event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
    let (event, _context) = event.into_parts();
    let user: &str;
    let token: &str;
    let mins: Vec<u32>;
    match event.query_string_parameters.first("user") {
        Some(usr) => user = usr,
        None => return Ok(missing_required_parameter("user")),
    }
    match event.query_string_parameters.first("token") {
        Some(t) => token = t,
        None => return Ok(missing_required_parameter("token")),
    }
    match event.query_string_parameters.all("min") {
        Some(min) => mins = min.iter().map(|m| m.parse::<u32>().unwrap()).collect(),
        None => return Ok(missing_required_parameter("min")),
    }
    let start = std::time::Instant::now();
    let ics = reqwest::get(format!(
        "https://outlook.office365.com/owa/calendar/{user}/{token}/calendar.ics"
    ))
    .await?
    .text()
    .await?;
    log::info!(
        "Office365 response time: {} ms",
        start.elapsed().as_secs_f64() * 1000.0
    );
    let body = add_alerts(ics, mins);
    let mut headers = HeaderMap::new();
    headers.append("Content-Type", "text/calendar".parse()?);
    Ok(ApiGatewayV2httpResponse {
        status_code: 200,
        headers,
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Text(body)),
        is_base64_encoded: Some(false),
        cookies: vec![],
    })
}

fn add_alerts(ics: String, mins: Vec<u32>) -> String {
    let mut alerts = mins
        .iter()
        .map(|m| {
            format!(
                "BEGIN:VALARM\n\
                TRIGGER:-PT{m}M\n\
                ACTION:DISPLAY\n\
                END:VALARM\n"
            )
        })
        .collect::<Vec<String>>();
    alerts.push(String::from("END:VEVENT"));
    let alerts = alerts.concat();
    ics.lines()
        .map(|line| {
            // Want to add alarm(s) just before end of event object
            if line == "END:VEVENT" {
                alerts.as_ref()
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

fn missing_required_parameter(param: &str) -> ApiGatewayV2httpResponse {
    ApiGatewayV2httpResponse {
        status_code: 400,
        headers: HeaderMap::new(),
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Text(format!(
            "{{\"error\": \"Missing required parameter: '{param}'\"}}"
        ))),
        is_base64_encoded: Some(false),
        cookies: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_alerts() {
        let mins = vec![5];
        let fake_ics = String::from(
            "BEGIN:VCALENDAR\n\
            BEGIN:VEVENT\n\
            END:VEVENT\n\
            END:VCALENDAR",
        );
        let expected = String::from(
            "BEGIN:VCALENDAR\n\
            BEGIN:VEVENT\n\
            BEGIN:VALARM\n\
            TRIGGER:-PT5M\n\
            ACTION:DISPLAY\n\
            END:VALARM\n\
            END:VEVENT\n\
            END:VCALENDAR",
        );
        let new_ics = add_alerts(fake_ics, mins);
        assert_eq!(new_ics, expected);
    }

    #[test]
    fn test_add_alerts_multiple_times() {
        let mins = vec![10, 15];
        let fake_ics = String::from(
            "BEGIN:VCALENDAR\n\
            BEGIN:VEVENT\n\
            END:VEVENT\n\
            END:VCALENDAR",
        );
        let expected = String::from(
            "BEGIN:VCALENDAR\n\
            BEGIN:VEVENT\n\
            BEGIN:VALARM\n\
            TRIGGER:-PT10M\n\
            ACTION:DISPLAY\n\
            END:VALARM\n\
            BEGIN:VALARM\n\
            TRIGGER:-PT15M\n\
            ACTION:DISPLAY\n\
            END:VALARM\n\
            END:VEVENT\n\
            END:VCALENDAR",
        );
        let new_ics = add_alerts(fake_ics, mins);
        assert_eq!(new_ics, expected);
    }
}
