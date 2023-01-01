
use lazy_static::lazy_static;
use prometheus::{Encoder,TextEncoder,Gauge,HistogramVec, IntCounterVec,GaugeVec};
use prometheus::{
    register_gauge,register_histogram_vec,opts,labels,register_int_counter_vec,register_gauge_vec
}; 

lazy_static! {
    pub static ref NGINX_REQUEST_COUNTER:IntCounterVec = register_int_counter_vec!("nginx_request_count","nginx_request_count",&["path","status","method","device_type"]).unwrap();
    pub static ref NGINX_REQUEST_TIME:GaugeVec = register_gauge_vec!("nginx_request_time","nginx_request_time",&["path","status","method","device_type"]).unwrap();
    pub static ref NGINX_UPSTREAM_RESPONSE_TIME:GaugeVec = register_gauge_vec!("nginx_upstream_response_time","upstream_response_time",&["path","status","method","device_type"]).unwrap();

    static ref HTTP_BODY_GAUGE: Gauge = register_gauge!(opts!(
        "http_response_size_bytes",
        "Th HTTP response sizes in bytes.",
            labels! {"handler" => "all",}
     )).unwrap();
    static ref HTTP_REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "The HTTP request latencies in seconds.",
        &["handler"]
    ).unwrap();
}

pub async fn prometheus_metrics()->String{
    let encoder = TextEncoder::new();
    let timer = HTTP_REQ_HISTOGRAM.with_label_values(&["all"]).start_timer();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    HTTP_BODY_GAUGE.set(buffer.len() as f64);
    timer.observe_duration();
    std::str::from_utf8(&buffer).unwrap_or("").to_string()
}