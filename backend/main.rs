mod bridge;
mod control;
mod domain;
pub mod entity;
mod infrastructure;

use control::services::startup::StartupService;
use infrastructure::{logging::LoggingManager, websocket::start_metrics_broadcaster};
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging first
    LoggingManager::initialize();

    tracing::info!("Starting the Rext Server 🦖");

    println!("Quick test... trying email");
    let email = Message::builder()
        .from(Mailbox::new(Some("NoBody".to_owned()), "nobody@domain.tld".parse().unwrap()))
        .reply_to(Mailbox::new(Some("Yuin".to_owned()), "yuin@domain.tld".parse().unwrap()))
        .to(Mailbox::new(Some("Hei".to_owned()), "hei@domain.tld".parse().unwrap()))
        .subject("Happy new year")
        .header(ContentType::TEXT_PLAIN)
        .body(String::from("Be happy!"))
        .unwrap();

    let creds = Credentials::new("smtp_username".to_owned(), "smtp_password".to_owned());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {e:?}"),
    }


    // Initialize the application
    let db = StartupService::initialize().await?;

    // Run all services concurrently
    let _result = tokio::join!(
        StartupService::run_server(db),
        StartupService::run_job_queue_monitor(),
        StartupService::run_scheduler(),
        start_metrics_broadcaster()
    );

    Ok(())
}
