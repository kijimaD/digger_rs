use super::LogFragment;
use rltk::prelude::*;
use std::sync::Mutex;

pub type LogType = Mutex<Vec<Vec<LogFragment>>>;

lazy_static! {
    pub static ref FIELD_LOG: LogType = Mutex::new(Vec::new());
    pub static ref BATTLE_LOG: LogType = Mutex::new(Vec::new());
}

pub fn append_fragment(fragment: LogFragment) {
    FIELD_LOG.lock().unwrap().push(vec![fragment]);
}

pub fn append_entry(fragments: Vec<LogFragment>, log: &LogType) {
    log.lock().unwrap().push(fragments);
}

pub fn clear_log(log: &LogType) {
    log.lock().unwrap().clear();
}

pub fn log_display() -> TextBuilder {
    let mut buf = TextBuilder::empty();

    FIELD_LOG.lock().unwrap().iter().rev().take(12).for_each(|log| {
        log.iter().for_each(|frag| {
            buf.fg(frag.color);
            buf.line_wrap(&frag.text);
        });
        buf.ln();
    });

    buf
}

pub fn clone_log() -> Vec<Vec<crate::gamelog::LogFragment>> {
    FIELD_LOG.lock().unwrap().clone()
}

pub fn restore_log(log: &mut Vec<Vec<crate::gamelog::LogFragment>>) {
    FIELD_LOG.lock().unwrap().clear();
    FIELD_LOG.lock().unwrap().append(log);
}

pub fn print_log(log: &LogType, console: &mut Box<dyn Console>, pos: Point) {
    let mut y = pos.y;
    let mut x = pos.x;

    log.lock().unwrap().iter().rev().take(6).for_each(|log| {
        log.iter().for_each(|frag| {
            console.print_color(
                x,
                y,
                frag.color.to_rgba(1.0),
                RGB::named(rltk::BLACK).to_rgba(1.0),
                &frag.text,
            );
            x += frag.text.len() as i32;
            x += 1;
        });
        y += 1;
        x = pos.x;
    });
}
