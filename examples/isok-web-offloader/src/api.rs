use actix_web::{get, web, App, HttpResponse, HttpServer};
use isok_data::broker_rpc::{CheckJobStatus, CheckResult};
use prost::Message;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message as KafkaMessage};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

#[derive(serde::Serialize)]
pub struct EndpointAggregatedResult {
    pub id: String,
    pub successful_checks: u64,
    pub total_checks: u64,
    pub last_updated: u64,
    pub last_latency: Option<u64>,
}

#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("index.html"))
}

#[get("/stats")]
async fn get_health(
    results: web::Data<Arc<Mutex<HashMap<String, EndpointAggregatedResult>>>>,
) -> HttpResponse {
    let results = results.lock().unwrap();
    HttpResponse::Ok().json(&*results)
}

pub async fn run_offloader_api(bootstrap_servers: String) -> JoinHandle<Result<(), String>> {
    // Set up Kafka consumer
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "isok.agent.results")
        .set("bootstrap.servers", bootstrap_servers)
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Failed to create consumer");

    consumer
        .subscribe(&["isok.agent.results"])
        .expect("Failed to subscribe to topic");

    // Store aggregated results
    let results = Arc::new(Mutex::new(
        HashMap::<String, EndpointAggregatedResult>::new(),
    ));
    let results_clone = results.clone();

    // Spawn Kafka consumer task
    tokio::spawn(async move {
        loop {
            match consumer.recv().await {
                Ok(msg) => {
                    if let Some(payload) = msg.payload() {
                        let result = CheckResult::decode(payload).unwrap();
                        let mut results = results_clone.lock().unwrap();

                        results
                            .entry(result.id_ulid.clone())
                            .and_modify(|agg| {
                                agg.successful_checks +=
                                    match result.status == CheckJobStatus::Reachable as i32 {
                                        true => 1,
                                        false => 0,
                                    };
                                agg.total_checks += 1;
                                agg.last_latency = result.metrics.as_ref().and_then(|m| m.latency);
                                agg.last_updated = chrono::offset::Local::now().timestamp() as u64
                            })
                            .or_insert_with(|| EndpointAggregatedResult {
                                id: result.id_ulid.clone(),
                                successful_checks: match result.status
                                    == CheckJobStatus::Reachable as i32
                                {
                                    true => 1,
                                    false => 0,
                                },
                                last_latency: result.metrics.as_ref().and_then(|m| m.latency),
                                total_checks: 1,
                                last_updated: chrono::offset::Local::now().timestamp() as u64,
                            });
                    }
                }
                Err(e) => eprintln!("Error receiving message: {}", e),
            }
        }
    });

    tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(results.clone()))
                .service(index)
                .service(get_health)
        })
        .bind("127.0.0.1:8080")
        .expect("Failed to bind to port")
        .run()
        .await
        .map_err(|e| format!("Failed to start HTTP server: {:?}", e))
    })
}
