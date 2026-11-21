use std::sync::Mutex;

use chrono::{Duration, Local, NaiveDateTime, Timelike};
use dominio::portas::Relogio;

/// Relógio do computador.
#[derive(Debug, Default, Clone, Copy)]
pub struct RelogioSistema;

impl Relogio for RelogioSistema {
    fn agora(&self) -> NaiveDateTime {
        let agora = Local::now().naive_local();
        agora.with_nanosecond(0).unwrap_or(agora)
    }
}

/// Relógio parado em um momento fixo — usado nos testes para simular o tempo.
#[derive(Debug)]
pub struct RelogioFixo(Mutex<NaiveDateTime>);

impl RelogioFixo {
    pub fn em(momento: NaiveDateTime) -> Self {
        RelogioFixo(Mutex::new(momento))
    }

    pub fn avancar_dias(&self, dias: i64) {
        let mut agora = self.0.lock().expect("relógio");
        *agora += Duration::days(dias);
    }
}

impl Relogio for RelogioFixo {
    fn agora(&self) -> NaiveDateTime {
        *self.0.lock().expect("relógio")
    }
}
