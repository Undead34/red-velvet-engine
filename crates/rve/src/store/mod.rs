use redis::aio::MultiplexedConnection;

pub struct RedisStore {
  client: redis::Client,
}

impl RedisStore {
  pub fn new() -> Self {
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();

    Self { client }
  }

  pub async fn get_connection(&self) -> MultiplexedConnection {
    self.client.get_multiplexed_async_connection().await.unwrap()
  }
}
