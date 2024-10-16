use crate::types::{Data, Error};

pub async fn log_error(error: poise::FrameworkError<'_, Data, Error>) {
    println!("Error: {:?}", error);
}