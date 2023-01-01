use tokio::net::UdpSocket;
use serde::{Deserialize, Serialize};
use crate::prome;

pub async fn tt(udp_port:u16){
    let server = UdpSocket::bind(format!("0.0.0.0:{}",udp_port)).await.unwrap();
    let mut buffer = [0; 4096];
    loop {
        match server.recv_from(&mut buffer).await{
            Ok((size,_))=>{//address
                let str = std::str::from_utf8(&buffer[..size]).unwrap();
                log::trace!("recv：{}", str);
                let a = str.split("|||").collect::<Vec<&str>>()[1];
                match serde_json::from_str::<NginxLog>(a){
                    Ok(mut request)=>{
                        request.parse();
                        match request.request_obj{
                            Some(req)=>{
                                prome::NGINX_REQUEST_COUNTER.with_label_values(&[
                                    &req.path,
                                    &request.status,
                                    &req.method,
                                    &req.device_type
                                ]).inc();
                                prome::NGINX_REQUEST_TIME.with_label_values(&[
                                    &req.path,
                                    &request.status,
                                    &req.method,
                                    &req.device_type
                                ]).set(request.request_time.parse::<f64>().unwrap_or_default());
                                match request.upstream_response_time.parse::<f64>(){
                                    Ok(resp_time)=>{
                                        prome::NGINX_UPSTREAM_RESPONSE_TIME.with_label_values(&[
                                            &req.path,
                                            &request.status,
                                            &req.method,
                                            &req.device_type
                                        ]).set(resp_time);
                                    },
                                    Err(_)=>{}
                                }                                
                                log::debug!("{:?}",req);
                            },
                            None=>{}
                        }
                    },
                    Err(e)=>log::error!("{}----{}",a,e)
                };
            },
            Err(e)=>{log::error!("{}",e)}
        };
    }
}

#[derive(Serialize, Deserialize,Debug,Clone)]
struct NginxLog {
    remote_addr:String,
    request:String,
    status:String,
    http_user_agent:String,
    upstream_response_time:String,
    request_time:String,
    request_obj:Option<Request>,
}

#[derive(Serialize, Deserialize,Debug,Clone)]
struct Request{
    method:String,
    path:String,
    http_version:String,
    device_type:String
}

impl NginxLog{
    fn parse(&mut self){
        let request_list:Vec<&str> = self.request.split(" ").collect();
        if request_list.len()==3{
            self.request_obj = Some(Request {
                method: request_list[0].to_string(),
                path: request_list[1].to_string(),
                http_version: request_list[2].to_string(),
                device_type:device_type(&self.http_user_agent).to_string()
            })
        }
    }
}

fn device_type(agent:&str)->&'static str{
    if agent.contains("Android"){
        "Android"
    }else if agent.contains("iOS"){
        "iOS"
    }else{
        "other"
    }
}