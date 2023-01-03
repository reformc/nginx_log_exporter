use std::{collections::HashMap, sync::{Arc, Mutex}};

use tokio::net::UdpSocket;
use serde::{Deserialize, Serialize};
use chrono::prelude::*;
use xdb::{search_by_ip, searcher_init};

use crate::prome;

pub async fn tt(udp_port:u16,ip_file:&str){
    let request_tmp = TimeConsumingWithLable::new();
    let response_tmp = TimeConsumingWithLable::new();
    request_tmp.tmp();
    response_tmp.tmp();
    let server = UdpSocket::bind(format!("0.0.0.0:{}",udp_port)).await.unwrap();
    let mut buffer = [0; 4096];
    searcher_init(Some(ip_file.to_owned()));
    loop {
        match server.recv_from(&mut buffer).await{
            Ok((size,_))=>{//address
                let str = std::str::from_utf8(&buffer[..size]).unwrap();
                log::trace!("recv:{}", str);
                let a = str.split("|||").collect::<Vec<&str>>()[1];
                match serde_json::from_str::<NginxLog>(a){
                    Ok(mut request)=>{
                        request.parse();
                        match request.request_obj{
                            Some(req)=>{
                                let label = vec!(req.path.clone(),request.status.clone(),req.method.clone(),req.device_type.clone(),request.http_host.clone());
                                prome::NGINX_REQUEST_COUNTER.with_label_values(&[
                                    &req.path,
                                    &request.status,
                                    &req.method,
                                    &req.device_type,
                                    &request.http_host
                                ]).inc();
                                request_tmp.set(label.clone(), request.request_time.parse::<f64>().unwrap_or_default());
                                prome::NGINX_REQUEST_TIME.with_label_values(&[
                                    &req.path,
                                    &request.status,
                                    &req.method,
                                    &req.device_type,
                                    &request.http_host
                                ]).set(request_tmp.get(&label).unwrap_or_default());
                                match request.upstream_response_time.parse::<f64>(){
                                    Ok(resp_time)=>{
                                        response_tmp.set(label.clone(),resp_time);
                                        prome::NGINX_UPSTREAM_RESPONSE_TIME.with_label_values(&[
                                            &req.path,
                                            &request.status,
                                            &req.method,
                                            &req.device_type,
                                            &request.http_host
                                        ]).set(response_tmp.get(&label).unwrap_or_default());
                                    },
                                    Err(_)=>{}
                                }                                
                                log::debug!("{},{},{},{},{:?}",search_by_ip(&*request.remote_addr).unwrap(),request.http_host,request.request_time,request.upstream_response_time,req);
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
    http_host:String,
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
                path: request_list[1].split("?").collect::<Vec<&str>>()[0].to_string(),//get请求去除参数
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

struct TimeConsumingWithLable{
    labels:Arc<Mutex<HashMap<Vec<String>,TimeConsuming>>>
}

impl TimeConsumingWithLable{
    pub fn new()->TimeConsumingWithLable{
        TimeConsumingWithLable{
            labels:Arc::new(Mutex::new(HashMap::new()))
        }
    }

    fn set(&self,labels:Vec<String>,value:f64){
        let mut label = self.labels.lock().unwrap();
        match label.get(&labels){
            Some(a)=>{
                a.insert(value);
            },
            None=>{
                let t = TimeConsuming::new();
                t.insert(value);
                label.insert(labels, t);
            }
        }
    }

    fn tmp(&self){
        let map = self.labels.clone();
        tokio::spawn(async move{
            loop{
                tokio::time::sleep(tokio::time::Duration::from_secs(15)).await;
                {
                    let label = map.lock().unwrap();
                    for (_,value) in label.iter(){
                        let mut l = value.list.lock().unwrap();
                        l.retain(|(timestamp,_)|timestamp +15>Local::now().timestamp());
                    }
                }
            }
        });
    }

    fn get(&self,labels:&Vec<String>)->Option<f64>{
        let label = self.labels.lock().unwrap();
        match label.get(labels){
            Some(a)=>{
                Some(a.avg())
            },
            None=>{None}
        }
    }
}

struct TimeConsuming{
    list:Arc<Mutex<Vec<(i64,f64)>>>
}

impl TimeConsuming{
    fn new()->TimeConsuming{
        TimeConsuming {list:Arc::new(Mutex::new(vec!()))}
    }

    fn insert(&self,value:f64){
        let mut a = self.list.lock().unwrap();
        a.retain(|(timestamp,_)|timestamp +15>Local::now().timestamp());
        a.push((Local::now().timestamp(),value))
    }

    fn avg(&self)->f64{
        let a = self.list.lock().unwrap();
        let mut res:f64 = 0.0;
        if a.len()==0{
            0.0
        }else{
            for (_,c) in a.iter(){
                res += c;
            }
            res/(a.len() as f64)
        }
    }
}