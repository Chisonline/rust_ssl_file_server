use chrono::{DateTime, FixedOffset};
use env_logger::{
    fmt::style::{self, RgbColor, Style},
    Builder,
};
use std::io::Write;

pub fn log_init() {
    Builder::new()
        .format(|buf, record| {
            let mut style = Style::new().bold();

            match record.level() {
                log::Level::Error => {
                    style = style.fg_color(Some(style::Color::Rgb(RgbColor(196, 0, 0))));
                }
                log::Level::Warn => {
                    style = style.fg_color(Some(style::Color::Rgb(RgbColor(255, 145, 0))));
                }
                log::Level::Info => {
                    style = style.fg_color(Some(style::Color::Rgb(RgbColor(0, 196, 196))));
                }
                log::Level::Trace => {
                    style = style.fg_color(Some(style::Color::Rgb(RgbColor(128, 128, 128))));
                }
                log::Level::Debug => {
                    style = style.fg_color(Some(style::Color::Rgb(RgbColor(128, 196, 0))));
                }
            }
            let timestamp = buf.timestamp();
            let shanghai_zone = FixedOffset::east_opt(8 * 3600).unwrap();
            let time = DateTime::parse_from_rfc3339(&format!("{timestamp}"))
                .unwrap()
                .with_timezone(&shanghai_zone)
                .format("%Y-%m-%d %H:%M:%S");

            writeln!(
                buf,
                "{style}{:5}{style:#} [{time}]: {}",
                record.level(),
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Always)
        .init();
}

#[allow(unused)]
pub fn test_log() {
    use log::*;
    error!("error");
    warn!("warn");
    info!("info");
    debug!("debug");
    trace!("trace");
}
