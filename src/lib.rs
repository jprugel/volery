pub mod protocol;

use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use serde::{Deserialize, Serialize};
use smol::io::{AsyncReadExt, AsyncWriteExt};
use smol::net::TcpStream;
use std::collections::HashMap;
use std::marker::PhantomData;
use uuid::Uuid;
use crate::protocol::*;

#[derive(Default)]
pub struct NetworkPlugin<T, U> {
    request_data: PhantomData<T>,
    response_data: PhantomData<U>,
}

impl<T: Message, U: Message> Plugin for NetworkPlugin<T, U> {
    fn build(&self, app: &mut App) {
        app.insert_resource(Network::<T, U>::default())
            .add_systems(FixedUpdate, network_system::<T, U>);
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub struct Network<T: Message, U: Message> {
    requests: HashMap<Uuid, T>,
    responses: HashMap<Uuid, U>,
}

impl<T: Message, U: Message> Network<T, U> {
    pub fn send(&mut self, request: T) ->  Uuid {
        let values = self.requests
            .iter()
            .filter(|(_key, value)| **value == request)
            .nth(0);

        let id = match values {
            Some(v) => *(v.0),
            None => Uuid::new_v4()
        };

        self.requests.insert(id, request);
        id
    }

    pub fn read(&self, id: Uuid) -> Option<&U> {
        self.responses.get(&id)
    }
}

fn network_system<T: Message, U: Message>(
    network: ResMut<Network<T, U>>,
    tracked_requests: Local<Vec<Uuid>>,
) {
    let task_pool = AsyncComputeTaskPool::get();
    let mut network = network.clone();
    let mut tracked_requests = tracked_requests.clone();

    let task = task_pool.spawn(async move {
        let mut stream = TcpStream::connect("127.0.0.1:8080")
            .await
            .expect("Failed to connect to server");

        info!("{:?}", network);
        let to_do = network
            .requests
            .into_iter()
            .filter(|(k, _)| !tracked_requests.contains(&k))
            .collect::<HashMap<Uuid, T>>();

        to_do.keys().for_each(|key| tracked_requests.push(*key));
        info!("Todo: {:?}", to_do);

        for (key, value) in to_do.iter() {
            let request_length = bincode::serialized_size::<T>(&value).unwrap();
            dbg!(request_length);
            let total_length = HEADER_LENGTH as u16 + request_length as u16;
            dbg!(total_length);
            let header = Header::builder()
                .version(V0)
                .form(FORM_REQUEST)
                .length(total_length)
                .reserved(0)
                .build();

            println!("{:?}", header);
                
            let serialized = bincode::serialize(&value).unwrap();
            let _ = send_packet(&mut stream, header, &serialized).await;

            let mut buffer = [0 as u8; 1024];
            let header_length = stream.read(&mut buffer[..HEADER_LENGTH as usize]).await.expect("Failed to read to end of buffer");
            dbg!(header_length);
            info!("Stream: {:?}", buffer);
            let header = Header::from_bytes(&buffer[..header_length]);           
            info!("Deserialized: {:?}", header);
        }
    });

    bevy::tasks::block_on(task);
}

