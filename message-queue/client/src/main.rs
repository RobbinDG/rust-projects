use backend::protocol::message::{Message, MessagePayload, TTL};
use backend::protocol::queue_id::{NewQueueId, QueueFilter, QueueId, TopicLiteral};
use backend::protocol::request::{CreateQueue, Publish, Receive, Subscribe};
use backend::protocol::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::UserQueueProperties;
use backend::DisconnectedClient;
use rand::random;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::task::JoinSet;

#[derive(Debug, Serialize, Deserialize)]
struct NumberPair {
    left: u8,
    right: u8,
}

#[tokio::main]
async fn main() {
    let server = DisconnectedClient::new("127.0.0.1:1234");
    let mut server = match server.connect().await {
        Ok(client) => client,
        Err(_) => panic!("Failed to connect to server"),
    };

    server
        .transfer_admin_request(CreateQueue {
            queue_address: NewQueueId::Queue("unused_numbers_dlx".to_string()),
            properties: UserQueueProperties {
                is_dlx: true,
                dlx: None,
            },
        })
        .await
        .unwrap();

    server
        .transfer_admin_request(CreateQueue {
            queue_address: NewQueueId::Topic(
                "numbers".to_string(),
                Some(("inputs".to_string(), Some("pairs".to_string()))),
            ),
            properties: UserQueueProperties {
                is_dlx: false,
                dlx: None,
            },
        })
        .await
        .unwrap();

    server
        .transfer_admin_request(CreateQueue {
            queue_address: NewQueueId::Topic(
                "numbers".to_string(),
                Some(("outputs".to_string(), Some("sums".to_string()))),
            ),
            properties: UserQueueProperties {
                is_dlx: false,
                dlx: None,
            },
        })
        .await
        .unwrap();

    server
        .transfer_admin_request(CreateQueue {
            queue_address: NewQueueId::Topic(
                "numbers".to_string(),
                Some(("outputs".to_string(), Some("products".to_string()))),
            ),
            properties: UserQueueProperties {
                is_dlx: false,
                dlx: None,
            },
        })
        .await
        .unwrap();

    let mut set = JoinSet::new();
    set.spawn(async move {
        let server = DisconnectedClient::new("127.0.0.1:1234");
        let mut server = match server.connect().await {
            Ok(client) => client,
            Err(_) => panic!("Failed to connect to server"),
        };

        loop {
            let payload = NumberPair {
                left: random(),
                right: random(),
            };

            server
                .transfer_admin_request(Publish {
                    message: Message {
                        payload: MessagePayload::encode_blob(&payload).unwrap(),
                        routing_key: RoutingKey {
                            id: QueueId::Topic(
                                "numbers".to_string(),
                                "inputs".to_string(),
                                "pairs".to_string(),
                            ),
                            dlx: DLXPreference::Override(QueueId::Queue(
                                "unused_numbers_dlx".to_string(),
                            )),
                        },
                        ttl: TTL::Duration(Duration::from_secs(10)),
                    },
                })
                .await
                .unwrap()
                .unwrap();
            tokio::time::sleep(Duration::from_secs(rand::random_range(1..5))).await;
        }
    });

    set.spawn(async move {
        let server = DisconnectedClient::new("127.0.0.1:1234");
        let mut server = match server.connect().await {
            Ok(client) => client,
            Err(_) => panic!("Failed to connect to server"),
        };

        server
            .transfer_admin_request(Subscribe {
                queue: QueueFilter::Topic(
                    "numbers".to_string(),
                    TopicLiteral::Name("inputs".to_string()),
                    TopicLiteral::Name("pairs".to_string()),
                ),
            })
            .await
            .unwrap();

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let response = server.transfer_admin_request(Receive {}).await.unwrap();
            if let Some(response) = response {
                let numbers = response.payload.decode_blob::<NumberPair>().unwrap();
                let result = format!(
                    "{} + {} = {}",
                    numbers.left,
                    numbers.right,
                    numbers.left as u32 + numbers.right as u32
                );
                tokio::time::sleep(Duration::from_secs(rand::random_range(1..3))).await; // Delay
                server
                    .transfer_admin_request(Publish {
                        message: Message {
                            payload: MessagePayload::Text(result),
                            routing_key: RoutingKey {
                                id: QueueId::Topic(
                                    "numbers".to_string(),
                                    "outputs".to_string(),
                                    "sums".to_string(),
                                ),
                                dlx: DLXPreference::Override(QueueId::Queue(
                                    "unused_numbers_dlx".to_string(),
                                )),
                            },
                            ttl: TTL::Duration(Duration::from_secs(10)),
                        },
                    })
                    .await
                    .unwrap()
                    .unwrap();
            }
        }
    });

    set.spawn(async move {
        let server = DisconnectedClient::new("127.0.0.1:1234");
        let mut server = match server.connect().await {
            Ok(client) => client,
            Err(_) => panic!("Failed to connect to server"),
        };

        server
            .transfer_admin_request(Subscribe {
                queue: QueueFilter::Topic(
                    "numbers".to_string(),
                    TopicLiteral::Name("inputs".to_string()),
                    TopicLiteral::Name("pairs".to_string()),
                ),
            })
            .await
            .unwrap();

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let response = server.transfer_admin_request(Receive {}).await.unwrap();
            if let Some(response) = response {
                let numbers = response.payload.decode_blob::<NumberPair>().unwrap();
                let result = format!(
                    "{} * {} = {}",
                    numbers.left,
                    numbers.right,
                    numbers.left as u32 * numbers.right as u32
                );
                tokio::time::sleep(Duration::from_secs(rand::random_range(1..3))).await; // Delay
                server
                    .transfer_admin_request(Publish {
                        message: Message {
                            payload: MessagePayload::Text(result),
                            routing_key: RoutingKey {
                                id: QueueId::Topic(
                                    "numbers".to_string(),
                                    "outputs".to_string(),
                                    "products".to_string(),
                                ),
                                dlx: DLXPreference::Override(QueueId::Queue(
                                    "unused_numbers_dlx".to_string(),
                                )),
                            },
                            ttl: TTL::Duration(Duration::from_secs(10)),
                        },
                    })
                    .await
                    .unwrap()
                    .unwrap();
            }
        }
    });

    set.spawn(async move {
        let server = DisconnectedClient::new("127.0.0.1:1234");
        let mut server = match server.connect().await {
            Ok(client) => client,
            Err(_) => panic!("Failed to connect to server"),
        };

        server
            .transfer_admin_request(Subscribe {
                queue: QueueFilter::Topic(
                    "numbers".to_string(),
                    TopicLiteral::Name("outputs".to_string()),
                    TopicLiteral::Wildcard,
                ),
            })
            .await
            .unwrap();

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let response = server.transfer_admin_request(Receive {}).await.unwrap();
            if let Some(response) = response {
                if let MessagePayload::Text(result) = response.payload {
                    println!("{}", result);
                }
            }
        }
    });

    set.join_all().await;
}
