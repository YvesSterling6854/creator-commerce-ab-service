use creator_commerce_ab::{plan_from_flag, CreatorRequest, InfraiClient};
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let user_id = args.next().unwrap_or_default();
    let flag_key = args.next().unwrap_or_default();
    let title = args
        .next()
        .unwrap_or_else(|| "creator download".to_string());
    let client = match InfraiClient::from_env() {
        Ok(client) => client,
        Err(error) => {
            eprintln!("{error:?}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = client.user(&user_id) {
        eprintln!("{error:?}");
        return ExitCode::FAILURE;
    }
    let request = CreatorRequest {
        user_id,
        flag_key,
        title,
    };
    let flag_value = match client.flag_value(&request.flag_key, &request.user_id) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error:?}");
            return ExitCode::FAILURE;
        }
    };
    match plan_from_flag(request, &flag_value) {
        Ok(plan) => println!(
            "{}: {} | {}",
            plan.user_id, plan.variant, plan.subscriber_update
        ),
        Err(error) => {
            eprintln!("{error:?}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
