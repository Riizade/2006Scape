use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use simplelog::*;

pub fn initialize_logging(level: LevelFilter) {
    let multi = MultiProgress::new();
    LogWrapper::new(
        multi.clone(),
        TermLogger::new(
            level,
            Config::default(),
            TerminalMode::Stderr,
            ColorChoice::Auto,
        ),
    )
    .try_init()
    .unwrap();
}
