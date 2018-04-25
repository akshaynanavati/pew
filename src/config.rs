use clap::{App, Arg};

const DEFAULT_MIN_DURATION: &str = "1";
const DEFAULT_MIN_RUNS: &str = "8";

pub struct Config {
    pub filter: String,
    pub min_duration: u64,
    pub min_runs: u8,
}

fn create_config() -> Config {
    let app_config = App::new("pew-benchmark")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("filter")
                .short("f")
                .long("filter")
                .value_name("FILTER")
                .help("Only run benchmarks that contain this string")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("min_duration")
                .short("d")
                .long("min_duration")
                .value_name("RUN_UNTIL")
                .help("Run benchmarks till this time (in s) and then output average")
                .takes_value(true)
                .default_value(DEFAULT_MIN_DURATION),
        )
        .arg(
            Arg::with_name("min_runs")
                .short("r")
                .long("min_runs")
                .value_name("MIN_RUNS")
                .help("Run benchmarks till this at lesat this many runs")
                .takes_value(true)
                .default_value(DEFAULT_MIN_RUNS),
        )
        .get_matches();

    let filter = match app_config.value_of("filter") {
        None => "",
        Some(s) => s,
    };

    let min_duration = app_config
        .value_of("min_duration")
        .unwrap()
        .parse::<u64>()
        .unwrap() * 1_000_000_000;
    let min_runs = app_config
        .value_of("min_runs")
        .unwrap()
        .parse::<u8>()
        .unwrap();
    Config {
        filter: filter.to_string(),
        min_duration,
        min_runs,
    }
}

lazy_static! {
    static ref PEW_CONFIG: Config = create_config();
}

impl Config {
    pub fn get() -> &'static Config {
        return &PEW_CONFIG;
    }
}
