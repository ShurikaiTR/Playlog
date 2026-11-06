use std::time::Duration;

use anyhow::Context;
use serde::Serialize;

#[derive(Clone)]
pub enum Mailer {
    Resend {
        client: reqwest::Client,
        api_key: String,
        from: String,
    },
    /// Geliştirme: e-posta göndermez, içeriği günlüğe yazar.
    Log,
}

#[derive(Serialize)]
struct ResendEmail<'a> {
    from: &'a str,
    to: [&'a str; 1],
    subject: &'a str,
    text: &'a str,
}

impl Mailer {
    pub fn new(api_key: Option<String>, from: String) -> anyhow::Result<Self> {
        let Some(api_key) = api_key else {
            tracing::warn!("RESEND_API_KEY yok: e-postalar gönderilmeyecek, günlüğe yazılacak");
            return Ok(Self::Log);
        };
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self::Resend {
            client,
            api_key,
            from,
        })
    }

    /// Resend API: POST https://api.resend.com/emails
    pub async fn send(&self, to: &str, subject: &str, text: &str) -> anyhow::Result<()> {
        match self {
            Self::Log => {
                tracing::info!(%to, %subject, body = %text, "e-posta (geliştirme modu)");
                Ok(())
            }
            Self::Resend {
                client,
                api_key,
                from,
            } => {
                client
                    .post("https://api.resend.com/emails")
                    .bearer_auth(api_key)
                    .json(&ResendEmail {
                        from,
                        to: [to],
                        subject,
                        text,
                    })
                    .send()
                    .await
                    .context("Resend isteği başarısız")?
                    .error_for_status()
                    .context("Resend hata döndürdü")?;
                Ok(())
            }
        }
    }
}
