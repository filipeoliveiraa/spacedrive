#![allow(unused)]

mod prisma;
mod prisma_crdt;

use ::prisma_crdt::CRDTOperation;
use prisma::PrismaClient;
// use serde_json::json;

use crate::prisma_crdt::new_client;
use serde_json::json;

#[tokio::main]
async fn main() {
	let client = prisma::new_client().await.unwrap();

	let node_0 = client
		.node()
		.upsert(
			prisma::node::id::equals(vec![0]),
			(
				prisma::node::id::set(vec![0]),
				prisma::node::name::set("Node 0".to_string()),
				vec![],
			),
			vec![],
		)
		.exec()
		.await
		.unwrap();

	let node_1 = client
		.node()
		.upsert(
			prisma::node::id::equals(vec![1]),
			(
				prisma::node::id::set(vec![1]),
				prisma::node::name::set("Node 1".to_string()),
				vec![],
			),
			vec![],
		)
		.exec()
		.await
		.unwrap();

	// producer_example(client, node_0).await;
	consumer_example(client, node_1).await;
}

async fn producer_example(client: PrismaClient, node: prisma::node::Data) {
	let (client, mut op_receiver) = new_client(client, node.id.clone(), node.local_id).await;

	let task = tokio::spawn(async move {
		while let Some(op) = op_receiver.recv().await {
			println!("{}", serde_json::to_string_pretty(&op).unwrap());
		}
	});

	let location = client
		.location()
		.create(vec![0], "Location 0".to_string(), vec![])
		.exec()
		.await
		.unwrap();

	let data = client
		.file_path()
		.create(0, location.local_id, "File 0".to_string(), vec![])
		.exec()
		.await
		.unwrap();

	let file = client.file().create(vec![0], vec![]).exec().await.unwrap();

	client
		.file_path()
		.update(
			prisma::file_path::location_id_id(data.location_id, data.id),
			vec![prisma_crdt::file_path::SetParam::SetFileId(Some(
				file.local_id,
			))],
		)
		.exec()
		.await
		.unwrap();
}

async fn consumer_example(client: PrismaClient, node: prisma::node::Data) {
	let (client, mut op_receiver) = new_client(client, node.id.clone(), node.local_id).await;

	client
		._execute_operation(
			serde_json::from_value(json!({
				"n": [0],
				"t": 0,
				"m": "Location",
				"d": [{
					"c": {
						"id": [0],
						"name": "Location 0"
					}
				}]
			}))
			.unwrap(),
		)
		.await;

	client
		._execute_operation(
			serde_json::from_value(json!({
				"n": [0],
				"t": 0,
				"m": "FilePath",
				"d": [{
					"c": {
						"id": 0,
						"location_id": [0],
						"name": "File 0"
					}
				}]
			}))
			.unwrap(),
		)
		.await;

	client
		._execute_operation(
			serde_json::from_value(json!({
			  "n": [0],
			  "t": 0,
			  "r": [0],
			  "m": "File",
			  "c": {}
			}))
			.unwrap(),
		)
		.await;

	client
		._execute_operation(
			serde_json::from_value(json!({
				"n": [0],
				"t": 0,
				"m": "FilePath",
				"d": [{
					"u": {
						"id": 0,
						"location_id": [0],
						"_": [{
							"file_id": 1
						}]
					}
				}]
			}))
			.unwrap(),
		)
		.await;

	dbg!(client.location().find_many(vec![]).exec().await);
	dbg!(client.file_path().find_many(vec![]).exec().await);
	dbg!(client.file().find_many(vec![]).exec().await);
}
