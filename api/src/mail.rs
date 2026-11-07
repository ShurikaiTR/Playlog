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

/// Kod e-postalarının metinleri (dile göre).
pub struct CodeEmailCopy {
    pub lang: &'static str,
    pub subject: &'static str,
    pub title: &'static str,
    pub greeting: &'static str,
    /// `{ttl}` yerine geçerlilik süresi (dakika) yazılır.
    pub instruction: &'static str,
    pub ignore: &'static str,
    pub team: &'static str,
}

pub const LOGIN_CODE_COPY: CodeEmailCopy = CodeEmailCopy {
    lang: "en",
    subject: "Your Playthru login code",
    title: "Sign in to Playthru",
    greeting: "Hi,",
    instruction: "Use the code below to sign in. It expires in {ttl} minutes.",
    ignore: "If you didn't request this, you can ignore this email.",
    team: "Playthru Team",
};

pub const SIGNUP_CODE_COPY: CodeEmailCopy = CodeEmailCopy {
    lang: "tr",
    subject: "Playthru doğrulama kodun",
    title: "E-postanı doğrula",
    greeting: "Merhaba,",
    instruction: "Kaydını tamamlamak için aşağıdaki kodu kullan. Kod {ttl} dakika geçerlidir.",
    ignore: "Bu isteği sen yapmadıysan bu e-postayı yok sayabilirsin.",
    team: "Playthru Ekibi",
};

pub const RESET_CODE_COPY: CodeEmailCopy = CodeEmailCopy {
    lang: "tr",
    subject: "Playthru şifre sıfırlama kodun",
    title: "Şifreni sıfırla",
    greeting: "Merhaba,",
    instruction: "Şifreni sıfırlamak için aşağıdaki kodu kullan. Kod {ttl} dakika geçerlidir.",
    ignore: "Bu isteği sen yapmadıysan bu e-postayı yok sayabilirsin.",
    team: "Playthru Ekibi",
};

/// Kodu 123-456 biçiminde gösterir. `code` yalnızca rakamdır (`code::generate`).
fn grouped(code: &str) -> String {
    match code.len() {
        6 => format!("{}-{}", &code[..3], &code[3..]),
        _ => code.to_owned(),
    }
}

/// Kod e-postası: (konu, düz metin, HTML). HTML tablo tabanlı ve satır içi stillidir
/// (e-posta istemcileri `<style>` ve modern CSS'i güvenilir uygulamaz).
pub fn code_email(copy: &CodeEmailCopy, code: &str, ttl_minutes: i64) -> (String, String, String) {
    let shown = grouped(code);
    let instruction = copy.instruction.replace("{ttl}", &ttl_minutes.to_string());
    let subject = format!("{}: {code}", copy.subject);

    let text = format!(
        "{}\n\n{instruction}\n\n{shown}\n\n{}\n\n{}\n",
        copy.greeting, copy.ignore, copy.team
    );

    const FONT: &str =
        "-apple-system,system-ui,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif";
    let html = format!(
        r##"<!doctype html>
<html lang="{lang}">
  <body style="margin:0;padding:0;background:#ffffff;">
    <table role="presentation" width="100%" border="0" cellpadding="0" cellspacing="0">
      <tr><td align="center">
        <table role="presentation" width="100%" border="0" cellpadding="0" cellspacing="0" style="max-width:600px;">
          <tr><td align="left" style="padding:10px 25px;"><div style="font-family:{FONT};font-size:24px;line-height:24px;font-weight:700;color:#111111;">Playthru</div></td></tr>
          <tr><td align="left" style="padding:24px 25px;">
            <div style="font-family:{FONT};font-size:15px;font-weight:300;line-height:24px;color:#333333;">
              <h1 style="font-size:22px;line-height:28px;font-weight:600;margin:0 0 16px;">{title}</h1>
              <p style="margin:0 0 16px;">{greeting}</p>
              <p style="margin:0 0 16px;">{instruction}</p>
              <table role="presentation" width="100%" border="0" cellpadding="0" cellspacing="0">
                <tr><td style="height:15px;"></td></tr>
                <tr><td style="text-align:center;background:#fafafa;padding:30px 20px;font-size:26px;font-weight:bold;">{shown}</td></tr>
                <tr><td style="height:15px;"></td></tr>
              </table>
              <p style="margin:16px 0 0;">{ignore}</p>
            </div>
          </td></tr>
          <tr><td align="left" style="padding:10px 25px;"><div style="font-family:{FONT};font-size:15px;font-weight:300;line-height:24px;color:#333333;">{team}</div></td></tr>
        </table>
      </td></tr>
    </table>
  </body>
</html>"##,
        lang = copy.lang,
        title = copy.title,
        greeting = copy.greeting,
        ignore = copy.ignore,
        team = copy.team,
    );
    (subject, text, html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_is_grouped_as_three_dash_three() {
        assert_eq!(grouped("042917"), "042-917");
    }

    #[test]
    fn login_code_email_has_the_code_ttl_and_ignore_line() {
        let (subject, text, html) = code_email(&LOGIN_CODE_COPY, "042917", 10);
        assert!(subject.contains("042917"));
        assert!(text.starts_with("Hi,"));
        for body in [&text, &html] {
            assert!(body.contains("042-917"));
            assert!(body.contains("expires in 10 minutes"));
            assert!(body.contains("you can ignore this email"));
            assert!(body.contains("Playthru Team"));
            assert!(!body.contains("2026"));
        }
    }
}
