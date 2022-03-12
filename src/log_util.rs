use std::io::Write;
use chrono::Local;
use env_logger::Builder;
use log::{LevelFilter, Level};
use env_logger::fmt::Color;

pub fn init_log () {
  // for log format
  Builder::new()
    .format(|buf, record| {
      let level = record.level();
      let c;
      match level {
        Level::Error => {
          c = Color::Red
        }
        Level::Info => {
          c = Color::Blue
        }
        Level::Warn => {
          c = Color::Yellow
        }
        _ => c = Color::White
      }
      let mut level_style = buf.style();
      level_style.set_color(c).set_bold(true);
      writeln!(
        buf,
        "{} [{}] - {}",
        level_style.value(Local::now().format("%Y-%m-%dT%H:%M:%S")),
        level_style.value(record.level()),
        level_style.value(record.args())
      )
    })
    .filter(None, LevelFilter::Info)
    .init();
}