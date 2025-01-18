use crate::tzdb::FsTzdbProvider;
use std::sync::{LazyLock, Mutex};

pub static TZ_PROVIDER: LazyLock<Mutex<FsTzdbProvider>> =
    LazyLock::new(|| Mutex::new(FsTzdbProvider::default()));
