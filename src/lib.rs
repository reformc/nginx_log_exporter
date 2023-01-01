mod syslog;
mod prome;
mod web;
use clap::Parser;

#[derive(Parser)]
#[clap(
    author="reform <reformgg@gmail.com>", 
    version="0.1.0",
    about="nginx日志转prometheus",
    long_about = "nginx日志转prometheus"
)]
struct Args{
    /// 监听syslog端口
    #[clap(long,short,default_value = "11514")]
    udp_port: u16,
    /// metrics端口
    #[clap(long,short,default_value = "80")]
    web_port: u16,
    ///日志级别,trace,debug,info,warn,error五种级别，默认为info
    #[clap(long,short,default_value = "info")]
    log_level: String
}

#[tokio::main]
pub async fn run(){
    let args = Args::parse();
    match &args.log_level as &str{
        "trace"=>simple_logger::init_with_level(log::Level::Trace).unwrap(),
        "debug"=>simple_logger::init_with_level(log::Level::Debug).unwrap(),
        "info"=>simple_logger::init_with_level(log::Level::Info).unwrap(),
        "warn"=>simple_logger::init_with_level(log::Level::Warn).unwrap(),
        _=>simple_logger::init_with_level(log::Level::Error).unwrap()
    }
    tokio::spawn(web::run(args.web_port));
    syslog::tt(args.udp_port).await;
}