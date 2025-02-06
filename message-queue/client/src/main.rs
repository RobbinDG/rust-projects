use backend::protocol::message::{Message, MessagePayload, TTL};
use backend::protocol::queue_id::{NewQueueId, QueueFilter, QueueId, TopicLiteral};
use backend::protocol::request::{CreateQueue, Publish, Receive, Subscribe};
use backend::protocol::routing_key::{DLXPreference, RoutingKey};
use backend::protocol::UserQueueProperties;
use backend::DisconnectedClient;
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
                "inputs".to_string(),
                Some(("pairs".to_string(), Some("numbers".to_string()))),
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
                left: 2,
                right: 3,
            };

            server.transfer_admin_request(Publish {
                message: Message {
                    payload: MessagePayload::encode_blob(&payload).unwrap(),
                    routing_key: RoutingKey {
                        id: QueueId::Topic(
                            "inputs".to_string(),
                            "pairs".to_string(),
                            "numbers".to_string(),
                        ),
                        dlx: DLXPreference::Override(QueueId::Queue(
                            "unused_numbers_dlx".to_string(),
                        )),
                    },
                    ttl: TTL::Duration(Duration::from_secs(10)),
                },
            }).await.unwrap().unwrap();
            tokio::time::sleep(Duration::from_secs(5)).await;
        };
    });

    set.spawn(async move {
        let server = DisconnectedClient::new("127.0.0.1:1234");
        let mut server = match server.connect().await {
            Ok(client) => client,
            Err(_) => panic!("Failed to connect to server"),
        };

        server.transfer_admin_request(Subscribe {
            queue: QueueFilter::Topic(
                "inputs".to_string(),
                TopicLiteral::Name("pairs".to_string()),
                TopicLiteral::Name("numbers".to_string()),
            )
        }).await.unwrap();

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            let response = server.transfer_admin_request(Receive {}).await.unwrap();
            if let Some(response) = response {
                println!("{:?}", response.payload.decode_blob::<NumberPair>().unwrap());
            }
        };
    });

    set.join_all().await;
}
