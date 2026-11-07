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
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<&'a str>,
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
    /// `html` isteğe bağlıdır; verilirse `text` düz metin alternatifi olarak da gider.
    pub async fn send(
        &self,
        to: &str,
        subject: &str,
        text: &str,
        html: Option<&str>,
    ) -> anyhow::Result<()> {
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
                        html,
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

/// Giriş kodu e-postası (İngilizce): konu, düz metin ve HTML gövde.
/// `code` yalnızca rakamdır (`code::generate`), bu yüzden HTML'e kaçışsız girer.
pub fn login_code_email(code: &str, ttl_minutes: i64) -> (String, String, String) {
    let subject = format!("Your Playthru login code: {code}");
    let text = format!(
        "Hi,\n\nYour Playthru login code is:\n\n{code}\n\n\
         It expires in {ttl_minutes} minutes. If you didn't request this, you can ignore this email."
    );
    let html = format!(
        r#"<!doctype html>
<html lang="en">
  <body style="margin:0;padding:24px;background:#ffffff;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;color:#111111;">
    <div style="max-width:420px;margin:0 auto;">
      <p style="margin:0 0 16px;font-size:16px;line-height:24px;">Hi,</p>
      <p style="margin:0 0 20px;font-size:16px;line-height:24px;">Use this code to log in to Playthru:</p>
      <p style="margin:0 0 20px;font-size:36px;line-height:44px;font-weight:600;letter-spacing:8px;font-family:ui-monospace,SFMono-Regular,Menlo,Consolas,monospace;">{code}</p>
      <p style="margin:0 0 8px;font-size:14px;line-height:20px;color:#555555;">This code expires in {ttl_minutes} minutes.</p>
      <p style="margin:0;font-size:14px;line-height:20px;color:#555555;">If you didn't request this, you can safely ignore this email.</p>
    </div>
  </body>
</html>"#
    );
    (subject, text, html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_code_email_has_the_code_ttl_and_ignore_line() {
        let (subject, text, html) = login_code_email("042917", 10);
        assert!(subject.contains("042917"));
        for body in [&text, &html] {
            assert!(body.starts_with("Hi,") || body.contains(">Hi,<"));
            assert!(body.contains("042917"));
            assert!(body.contains("10 minutes"));
            assert!(body.contains("ignore this email"));
        }
    }
}
