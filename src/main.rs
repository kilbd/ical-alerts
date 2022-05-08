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
    log::info!("user={user}, token={token}, mins={mins:?}");
    Ok(ApiGatewayV2httpResponse {
        status_code: 200,
        headers: HeaderMap::new(),
        multi_value_headers: HeaderMap::new(),
        body: Some(Body::Text(String::from(""))),
        is_base64_encoded: Some(false),
        cookies: vec![],
    })
}

fn add_alerts(ics: String) -> String {
    ics.lines()
        .map(|line| {
            // Want to add alarm(s) just before end of event object
            if line == "END:VEVENT" {
                "BEGIN:VALARM\n\
                TRIGGER:-PT5M\n\
                ACTION:DISPLAY\n\
                END:VALARM\n\
                END:VEVENT"
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
        let new_ics = add_alerts(fake_ics);
        assert_eq!(new_ics, expected);
    }
}
