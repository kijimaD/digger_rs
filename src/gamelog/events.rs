//! 累計実績の数値を保存するのに使う。
//! 例: 受けた+与えたダメージの合計、倒した敵の数、踏破した階層の数、...

use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref EVENTS: Mutex<HashMap<String, i32>> = Mutex::new(HashMap::new());
}

pub fn clear_events() {
    EVENTS.lock().unwrap().clear();
}

pub fn record_event<T: ToString>(event: T, n: i32) {
    let event_name = event.to_string();
    let mut events_lock = EVENTS.lock();
    let events = events_lock.as_mut().unwrap();
    if let Some(e) = events.get_mut(&event_name) {
        *e += n;
    } else {
        events.insert(event_name, n);
    }
}

pub fn get_event_count<T: ToString>(event: T) -> i32 {
    let event_name = event.to_string();
    let events_lock = EVENTS.lock();
    let events = events_lock.unwrap();
    if let Some(e) = events.get(&event_name) {
        *e
    } else {
        0
    }
}
