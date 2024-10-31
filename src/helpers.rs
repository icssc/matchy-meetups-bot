use crate::types::Data;
use anyhow::Error;

pub async fn log_error(error: poise::FrameworkError<'_, Data, Error>) {
    println!("Error: {:?}", error);
}
