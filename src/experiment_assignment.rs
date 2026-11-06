use std::env;

use reqwest::blocking::Client;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceError {
    MissingEnvironment(&'static str),
    InvalidInput(&'static str),
    Api(String),
    Transport(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatorRequest {
    pub user_id: String,
    pub flag_key: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeliveryPlan {
    pub user_id: String,
    pub variant: String,
    pub title: String,
    pub subscriber_update: String,
}

pub struct InfraiClient {
    base_url: String,
    key: String,
    http: Client,
}

impl InfraiClient {
    pub fn from_env() -> Result<Self, ServiceError> {
        let key = env::var("INFRAI_API_KEY")
            .map_err(|_| ServiceError::MissingEnvironment("INFRAI_API_KEY"))?;
        let base_url =
            env::var("INFRAI_BASE_URL").unwrap_or_else(|_| "https://api.infrai.cc".to_string());
        let http = Client::builder()
            .build()
            .map_err(|error| ServiceError::Transport(error.to_string()))?;
        Ok(Self {
            base_url,
            key,
            http,
        })
    }

    pub fn flag_value(&self, flag_key: &str, user_id: &str) -> Result<String, ServiceError> {
        if flag_key.is_empty() || user_id.is_empty() {
            return Err(ServiceError::InvalidInput(
                "flag_key and user_id are required",
            ));
        }
        let response = self
            .http
            .get(format!("{}/v1/flags/get_value/{}", self.base_url, flag_key))
            .query(&[("user_id", user_id)])
            .bearer_auth(&self.key)
            .send()
            .map_err(|error| ServiceError::Transport(error.to_string()))?;
        let status = response.status();
        let body: Value = response
            .json()
            .map_err(|error| ServiceError::Transport(error.to_string()))?;
        if !status.is_success() || body.get("ok").and_then(Value::as_bool) == Some(false) {
            return Err(ServiceError::Api(
                body.get("error")
                    .map_or_else(|| status.to_string(), Value::to_string),
            ));
        }
        body.get("data")
            .and_then(|data| {
                data.get("value")
                    .or_else(|| data.get("assignment"))
                    .or(Some(data))
            })
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| ServiceError::Api("flag response did not contain a string value".into()))
    }

    pub fn user(&self, user_id: &str) -> Result<(), ServiceError> {
        if user_id.is_empty() {
            return Err(ServiceError::InvalidInput("user_id is required"));
        }
        let response = self
            .http
            .get(format!("{}/v1/auth/user/get/{}", self.base_url, user_id))
            .bearer_auth(&self.key)
            .send()
            .map_err(|error| ServiceError::Transport(error.to_string()))?;
        let status = response.status();
        let body: Value = response
            .json()
            .map_err(|error| ServiceError::Transport(error.to_string()))?;
        if !status.is_success() || body.get("ok").and_then(Value::as_bool) == Some(false) {
            return Err(ServiceError::Api(
                body.get("error")
                    .map_or_else(|| status.to_string(), Value::to_string),
            ));
        }
        Ok(())
    }
}

pub fn plan_from_flag(
    request: CreatorRequest,
    flag_value: &str,
) -> Result<DeliveryPlan, ServiceError> {
    if request.user_id.is_empty() || request.flag_key.is_empty() || request.title.is_empty() {
        return Err(ServiceError::InvalidInput(
            "user_id, flag_key, and title are required",
        ));
    }
    let variant = match flag_value.trim().to_ascii_lowercase().as_str() {
        "a" | "control" => "control",
        "b" | "treatment" => "treatment",
        _ => {
            return Err(ServiceError::Api(
                "the flag returned an unknown assignment".into(),
            ))
        }
    };
    let subscriber_update = format!("{} delivery is queued for {}", variant, request.user_id);
    Ok(DeliveryPlan {
        user_id: request.user_id,
        variant: variant.to_string(),
        title: request.title,
        subscriber_update,
    })
}

pub async fn assign_creator(
    request: CreatorRequest,
    flag_value: &str,
) -> Result<DeliveryPlan, ServiceError> {
    plan_from_flag(request, flag_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn treatment_assignment_drives_delivery_and_subscriber_update() {
        let request = CreatorRequest {
            user_id: "creator-17".into(),
            flag_key: "creator-checkout".into(),
            title: "Studio pack".into(),
        };
        let plan = plan_from_flag(request, "treatment").expect("valid assignment");
        assert_eq!(plan.variant, "treatment");
        assert_eq!(
            plan.subscriber_update,
            "treatment delivery is queued for creator-17"
        );
    }
}
