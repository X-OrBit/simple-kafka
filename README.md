# 🚤 Simple kafka
### Message broker

A service for registering publishers, subscribers, connecting them to topics and providing a broker for sending messages from publishers to subscribers

Project on the HSE Rust course

## 🔧 Installation

You must have the project [dependencies](#-dependencies) installed

1. Cloning a repository

```shell
git clone https://github.com/X-OrBit/simple-kafka
```

2. Going to the simple-kafka directory

```shell
cd simple-kafka
```

3. Building

```shell
cargo build -p simple-kafka -r
```

The binary file will be located along the path `./target/release/simple-kafka`

## 📦 Releases

Releases and builds of the program can be found at the [link](https://github.com/X-OrBit/simple-kafka/releases)

## 👔 Dependencies

For this project you must have installed Rust compiler and cargo:

### MacOS

#### Homebrew
```shell
sudo brew install rust
```

#### MacPorts
```shell
sudo port install rust
```

### MacOS, Linux and other Unix-like OS

Run the following in terminal, then follow the on-screen instructions:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Windows

Download and run [rustup-init.exe](https://static.rust-lang.org/rustup/dist/i686-pc-windows-gnu/rustup-init.exe)


## 🚀 Usage

### Server

Start server:

```shell
simple-kafka --address 127.0.0.1 --port 27727
```

### Client

#### Publisher

Connect to server

```shell
nc 127.0.0.1 27727
```

Send connection message line:
```
{"method": "publish", "topic": "<topic_name>"}
```

Send message. Each line is one message

```
Message to topic <topic_name>
```

#### Subscriber

Connect to server

```shell
nc 127.0.0.1 27727
```

Send connection message line:
```
{"method": "subscribe", "topic": "<topic_name>"}
```

Now client listening messages from topic `<topic_name>`


## ☑️ TODO list
- [ ] Prohibit topics that do not match the pattern: `[a-zA-Z_-0-9]{3,64}`
- [ ] Assign each connection `connection_id`
- [ ] Add unique name support for publishers and subscribers

```json
{
  "method": "publish",
  "...": "...",
  "name": "<unique_name>"
}
```

- [ ] Add support subscribe and publishing to multiple topics

```json
{
  "method": "publish/subscribe",
  "...": "...",
  "name": "<unique_name>"
}
```

- [ ] Make messages in json format. Accept publishing only in JSON format

Publish message method:

```json
{
  "method": "send_message",
  "text": "<message_text>"
}
```

Received message:

```json
{
  "type": "message",
  "data": {
    "publisher_name": "Admin publisher",
    "publisher_id": "7776969",
    "from_topic": "test_topic",
    "message_text": "Test message. Trying out the possibilities",
    "time": 1688720400 
  }
}
```

- [ ] Allow multiline JSON messages

- [ ] Sending service message to publishers/subscribers, when connecting to server and when other publisher/subscriber connected:

```json
{
  "type": "connection_info",
  "data": {
    "me": {
      "id": 7776969,
      "name": "Admin publisher",
      "role": "publisher"
    },
    "topics": {
      "topic1": {
        "publishers": [
          {
            "id": 7776968,
            "name": "Admin publisher (test)"
          }
        ],
        "subscribers": [
          {
            "id": 1234567,
            "name": "Test subscriber"
          }
        ]
      },
      "topic2": {
        "publishers": [],
        "subscribers": []
      }
    },
    "time": 1688720400
  }
}
```

```json
{
  "type": "new_subscriber",
  "data": {
    "subscriber": {
      "id": 1234567,
      "name": "Test subscriber"
    },
    "connected_topics": ["topic_name"],
    "time": 1688720400
  }
}
```

