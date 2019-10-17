use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "sdot-service",
    author = "ChainX",
    about = "For mapping sdot"
)]

pub struct CliConfig {
    #[structopt(long = "rpc-port", value_name = "PORT", default_value = "8100")]
    pub rpc_port: u16,
}

pub fn init() -> CliConfig {
    CliConfig::from_args()
}

pub fn config_url(conf: &CliConfig) -> String {
    format!("0.0.0.0:{}", conf.rpc_port)
}