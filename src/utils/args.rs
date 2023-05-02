use clap::Parser;
use strum_macros::EnumString;

#[derive(Parser, Debug)]
#[command(author = "smsimone", version)]
pub struct Args {
    #[clap(index = 0)]
    pub sub_command: SubCommand,

    /// Path to the aab file that have to be installed on android devices
    #[arg(short, long)]
    pub aab_path: String,

    /// Path to the ipa file for the iOS devices
    #[arg(short, long)]
    pub ipa_path: String,
}

#[derive(EnumString, Clone, Debug)]
pub enum SubCommand {
    #[strum(serialize = "find_devices")]
    FindDevices,
}
