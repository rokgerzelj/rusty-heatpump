use bytes::Bytes;
use rumqttc::Incoming;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, QoS};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{error, info};
use uuid::Uuid;

pub struct MqttClient {
    client: AsyncClient,
    event_loop: EventLoop,
}

#[derive(Debug)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: Bytes,
}

// Small abstraction on top of rumqttc::EventLoop,
// to make mqtt messages easier to work with in the rest of the application.
impl MqttClient {
    pub fn new(mqtt_host: String, mqtt_port: u16) -> MqttClient {
        let client_id = Uuid::new_v4().to_string();
        let mqtt_options = MqttOptions::new(client_id, mqtt_host, mqtt_port)
            .set_max_packet_size(1 * 1024 * 1024, 1 * 1024 * 1024)
            .to_owned();

        let (client, event_loop) = AsyncClient::new(mqtt_options, 50);

        MqttClient { client, event_loop }
    }

    pub fn run(mut self) -> (JoinHandle<()>, mpsc::Receiver<MqttMessage>, AsyncClient) {
        let (tx, rx) = mpsc::channel(32);

        let handle = tokio::spawn(async move {
            loop {
                let event = self.event_loop.poll().await;

                match event {
                    Ok(event) => {
                        let result = MqttClient::handle_event(event);
                        if let Some(msg) = result {
                            if let Err(e) = tx.send(msg).await {
                                error!("cannot send to mqtt message channel: {:?}", e);
                                break; // break event loop if channel is closed
                            }
                        }
                    }
                    Err(e) => {
                        error!("rumqqtc polling error: {:?}", e);
                    }
                }
            }
        });

        (handle, rx, self.client)
    }

    fn handle_event(event: Event) -> Option<MqttMessage> {
        match event {
            Event::Incoming(Incoming::Publish(p)) => {
                info!("Topic: {}, Payload: {:?}", p.topic, p.payload);

                Option::Some(MqttMessage {
                    topic: p.topic,
                    payload: p.payload,
                })
            }
            _ => Option::None,
        }
    }

    pub async fn subscribe(&self, topic: &str, qos: QoS) -> Result<(), rumqttc::ClientError> {
        self.client.subscribe(topic, qos).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_event_publish() {
        let topic = "topic".to_string();
        let payload = Bytes::from("test payload");
        let p = rumqttc::Publish::new(topic.clone(), QoS::AtLeastOnce, payload.clone());
        let event = Event::Incoming(Incoming::Publish(p));

        let maybe_msg = MqttClient::handle_event(event);

        assert!(maybe_msg.is_some());
        let msg = maybe_msg.unwrap();
        assert_eq!(msg.topic, topic);
        assert_eq!(msg.payload, payload);
    }
}
