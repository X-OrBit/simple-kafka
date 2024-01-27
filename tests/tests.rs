mod message;
mod publisher;
mod subscriber;
pub mod utils;

use crate::message::Message;
use simple_kafka::Server;
use log::LevelFilter::Debug;
use publisher::Publisher;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use subscriber::{Subscriber, SubscriberTester};
use tokio::io;
use tokio::join;
use tokio::task::JoinHandle;
use tuple_conv::RepeatedTuple;

use message::MessageQueue;

mod integration_tests {
    use super::*;

    const LISTEN_WAIT_TIME: f64 = 5.0; // in seconds
    const DELIMITER: u8 = 10;

    fn setup_logs() {
        let _ = env_logger::builder()
            .filter_level(Debug)
            .is_test(true)
            .try_init();
    }

    async fn assert_join(results: Vec<JoinHandle<bool>>) {
        for result in results {
            let result = result.await;
            assert!(result.is_ok() && result.unwrap());
        }
    }

    fn gen_server_details() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0)
    }

    async fn create_server(server_details: SocketAddr) -> Server {
        Server::new(&server_details)
            .await
            .expect(format!("Unable to start server at {}", server_details).as_str())
    }
    async fn run_server(mut server: Server) -> JoinHandle<io::Result<()>> {
        tokio::spawn(async move { server.run().await })
    }

    async fn connect_subscriber(
        server_port: u16,
        topic: String,
        messages_queue: MessageQueue,
    ) -> JoinHandle<bool> {
        tokio::spawn(async move {
            let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), server_port);
            let subscriber = Subscriber::new(&socket_addr).await;
            let mut subscriber_tester = SubscriberTester::new(subscriber, DELIMITER);
            subscriber_tester
                .test_messages(topic, messages_queue, LISTEN_WAIT_TIME)
                .await
        })
    }

    async fn connect_publisher(
        server_port: u16,
        topic: String,
        messages_queue: MessageQueue,
    ) -> JoinHandle<bool> {
        tokio::spawn(async move {
            let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), server_port);
            let mut publisher = Publisher::new(&socket_addr).await;
            publisher
                .send_messages(topic, messages_queue, DELIMITER)
                .await
        })
    }

    #[tokio::test]
    async fn init_test() {
        setup_logs();
        let server_details = gen_server_details();
        let server = create_server(server_details).await;
        let server_task = run_server(server).await;
        server_task.abort();
    }

    #[tokio::test]
    async fn connection_test() {
        setup_logs();
        let server_details = gen_server_details();

        let server = create_server(server_details).await;
        let server_port = server.port();
        let server_task = run_server(server).await;

        let message_queue = MessageQueue::default();
        let topic = "test_topic".to_string();

        assert_join(
            join!(
                connect_publisher(server_port, topic.clone(), message_queue.clone()),
                connect_subscriber(server_port, topic.clone(), message_queue.clone()),
            )
            .to_vec(),
        )
        .await;

        server_task.abort();
    }

    #[tokio::test]
    async fn one_to_one_test() {
        setup_logs();
        let server_details = gen_server_details();

        let server = create_server(server_details).await;
        let server_port = server.port();
        let server_task = run_server(server).await;

        let message_queue = MessageQueue::new(vec![Message::new("test message".to_string(), 1.0)]);
        let topic = "test_topic".to_string();

        assert_join(
            join!(
                connect_publisher(server_port, topic.clone(), message_queue.clone()),
                connect_subscriber(server_port, topic.clone(), message_queue.clone()),
            )
            .to_vec(),
        )
        .await;

        server_task.abort();
    }

    #[tokio::test]
    async fn many_to_one_test() {
        setup_logs();
        let server_details = gen_server_details();

        let server = create_server(server_details).await;
        let server_port = server.port();
        let server_task = run_server(server).await;

        let publisher1_message_queue = MessageQueue::new(vec![Message::new(
            "test message from publisher 1".to_string(),
            1.0,
        )]);
        let publisher2_message_queue = MessageQueue::new(vec![Message::new(
            "test message from publisher 2".to_string(),
            2.0,
        )]);
        let publisher3_message_queue = MessageQueue::new(vec![Message::new(
            "test message from publisher 3".to_string(),
            3.0,
        )]);
        let subscriber_message_queue = publisher1_message_queue
            .union(&publisher2_message_queue)
            .union(&publisher3_message_queue);
        let topic = "test_topic".to_string();

        assert_join(
            join!(
                connect_publisher(server_port, topic.clone(), publisher1_message_queue),
                connect_publisher(server_port, topic.clone(), publisher2_message_queue),
                connect_publisher(server_port, topic.clone(), publisher3_message_queue),
                connect_subscriber(server_port, topic.clone(), subscriber_message_queue.clone()),
            )
            .to_vec(),
        )
        .await;

        server_task.abort();
    }

    #[tokio::test]
    async fn one_to_many_test() {
        setup_logs();
        let server_details = gen_server_details();

        let server = create_server(server_details).await;
        let server_port = server.port();
        let server_task = run_server(server).await;

        let message_queue = MessageQueue::new(vec![Message::new("test message".to_string(), 1.0)]);
        let topic = "test_topic".to_string();

        assert_join(
            join!(
                connect_publisher(server_port, topic.clone(), message_queue.clone()),
                connect_subscriber(server_port, topic.clone(), message_queue.clone()),
                connect_subscriber(server_port, topic.clone(), message_queue.clone()),
                connect_subscriber(server_port, topic.clone(), message_queue.clone()),
            )
            .to_vec(),
        )
        .await;

        server_task.abort();
    }

    #[tokio::test]
    async fn many_to_many_test() {
        setup_logs();
        let server_details = gen_server_details();

        let server = create_server(server_details).await;
        let server_port = server.port();
        let server_task = run_server(server).await;

        let publisher1_message_queue = MessageQueue::new(vec![Message::new(
            "test message from publisher 1".to_string(),
            1.0,
        )]);
        let publisher2_message_queue = MessageQueue::new(vec![Message::new(
            "test message from publisher 2".to_string(),
            2.0,
        )]);
        let publisher3_message_queue = MessageQueue::new(vec![Message::new(
            "test message from publisher 3".to_string(),
            3.0,
        )]);
        let subscriber_message_queue = publisher1_message_queue
            .union(&publisher2_message_queue)
            .union(&publisher3_message_queue);

        let topic = "test_topic".to_string();

        assert_join(
            join!(
                connect_publisher(server_port, topic.clone(), publisher1_message_queue),
                connect_publisher(server_port, topic.clone(), publisher2_message_queue),
                connect_publisher(server_port, topic.clone(), publisher3_message_queue),
                connect_subscriber(server_port, topic.clone(), subscriber_message_queue.clone()),
                connect_subscriber(server_port, topic.clone(), subscriber_message_queue.clone()),
                connect_subscriber(server_port, topic.clone(), subscriber_message_queue.clone()),
            )
            .to_vec(),
        )
        .await;

        server_task.abort();
    }

    #[tokio::test]
    async fn stress_test() {
        setup_logs();
        let server_details = gen_server_details();

        let server = create_server(server_details).await;
        let server_port = server.port();
        let server_task = run_server(server).await;

        let mut message_queue = MessageQueue::default();
        for i in 1..=100 {
            message_queue.push(Message::new(
                format!("test message {i}").to_string(),
                i as f64 / 10.0,
            ));
        }

        let topic = "test_topic".to_string();

        assert_join(
            join!(
                connect_publisher(server_port, topic.clone(), message_queue.clone()),
                connect_subscriber(server_port, topic.clone(), message_queue.clone()),
            )
            .to_vec(),
        )
        .await;

        server_task.abort();
    }

    #[tokio::test]
    async fn multiple_topic_test() {
        setup_logs();
        let server_details = gen_server_details();

        let server = create_server(server_details).await;
        let server_port = server.port();
        let server_task = run_server(server).await;

        let pub_t1_message_queue = MessageQueue::new(vec![
            Message::new("topic 1 message 1".to_string(), 1.0),
            Message::new("topic 1 message 3".to_string(), 3.0),
        ]);
        let pub2_t1_message_queue = MessageQueue::new(vec![
            Message::new("topic 1 message 2".to_string(), 2.0),
            Message::new("topic 1 message 4".to_string(), 4.0),
        ]);

        let pub1_t2_message_queue = MessageQueue::new(vec![
            Message::new("topic 2 message 1".to_string(), 1.0),
            Message::new("topic 2 message 3".to_string(), 3.0),
        ]);
        let pub2_t2_message_queue = MessageQueue::new(vec![
            Message::new("topic 2 message 2".to_string(), 2.0),
            Message::new("topic 2 message 4".to_string(), 4.0),
        ]);

        let sub_t1_message_queue = pub_t1_message_queue.union(&pub2_t1_message_queue);
        let sub_t2_message_queue = pub1_t2_message_queue.union(&pub2_t2_message_queue);

        let topic1 = "topic 1".to_string();
        let topic2 = "topic 2".to_string();

        assert_join(
            join!(
                connect_publisher(server_port, topic1.clone(), pub_t1_message_queue),
                connect_publisher(server_port, topic2.clone(), pub1_t2_message_queue),
                connect_publisher(server_port, topic2.clone(), pub2_t2_message_queue),
                connect_publisher(server_port, topic1.clone(), pub2_t1_message_queue),
                connect_subscriber(server_port, topic1.clone(), sub_t1_message_queue.clone()),
                connect_subscriber(server_port, topic2.clone(), sub_t2_message_queue.clone()),
                connect_subscriber(server_port, topic2.clone(), sub_t2_message_queue.clone()),
                connect_subscriber(server_port, topic1.clone(), sub_t1_message_queue.clone()),
            )
            .to_vec(),
        )
        .await;

        server_task.abort();
    }
}
