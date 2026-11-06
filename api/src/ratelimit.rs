//! Bellek içi sabit pencereli hız sınırlayıcı (tek API örneği için yeterli).
//!
//! Anahtar örnekleri: `login:ip:1.2.3.4`, `login:id:gokhan`. Örnek sayısı birden
//! fazla olursa sayaçlar paylaşılmaz; o durumda Postgres tabanlı sayaca geçilir.

use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

/// Bellek şişmesini önlemek için tablo bu boyutu aşınca süresi dolanlar temizlenir.
const PRUNE_THRESHOLD: usize = 10_000;

#[derive(Default)]
pub struct RateLimiter {
    windows: Mutex<HashMap<String, Window>>,
}

struct Window {
    started: Instant,
    count: u32,
}

impl RateLimiter {
    /// Anahtar için bir deneme sayar. Sınır aşıldıysa kalan bekleme süresini döner.
    pub fn hit(&self, key: &str, max: u32, window: Duration) -> Result<(), Duration> {
        let now = Instant::now();
        // Zehirli kilit durumunda sayaçlar yine de kullanılabilir.
        let mut windows = self
            .windows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if windows.len() >= PRUNE_THRESHOLD {
            windows.retain(|_, w| now.duration_since(w.started) < window);
        }

        let entry = windows.entry(key.to_owned()).or_insert(Window {
            started: now,
            count: 0,
        });
        if now.duration_since(entry.started) >= window {
            *entry = Window {
                started: now,
                count: 0,
            };
        }
        if entry.count >= max {
            return Err(window.saturating_sub(now.duration_since(entry.started)));
        }
        entry.count += 1;
        Ok(())
    }

    /// Başarılı işlemden sonra sayacı sıfırlar (örn. doğru parola ile giriş).
    pub fn reset(&self, key: &str) {
        self.windows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_max_and_reports_wait() {
        let limiter = RateLimiter::default();
        let window = Duration::from_secs(60);
        for _ in 0..3 {
            assert!(limiter.hit("k", 3, window).is_ok());
        }
        let wait = limiter.hit("k", 3, window).unwrap_err();
        assert!(wait <= window && wait > Duration::ZERO);
        assert!(limiter.hit("other", 3, window).is_ok());
    }

    #[test]
    fn window_expires_and_reset_clears() {
        let limiter = RateLimiter::default();
        let window = Duration::from_millis(20);
        assert!(limiter.hit("k", 1, window).is_ok());
        assert!(limiter.hit("k", 1, window).is_err());
        std::thread::sleep(Duration::from_millis(30));
        assert!(limiter.hit("k", 1, window).is_ok());
        limiter.reset("k");
        assert!(limiter.hit("k", 1, Duration::from_secs(60)).is_ok());
    }
}
