use crate::{
	prisma::{self, node},
	Node,
};
use chrono::{DateTime, Utc};
use int_enum::IntEnum;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use ts_rs::TS;

mod config;

pub use config::*;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct LibraryNode {
	pub uuid: String,
	pub name: String,
	pub platform: Platform,
	pub tcp_address: String,
	#[ts(type = "string")]
	pub last_seen: DateTime<Utc>,
	#[ts(type = "string")]
	pub last_synchronized: DateTime<Utc>,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, Eq, PartialEq, IntEnum)]
#[ts(export)]
pub enum Platform {
	Unknown = 0,
	Windows = 1,
	MacOS = 2,
	Linux = 3,
	IOS = 4,
	Android = 5,
}

impl LibraryNode {
	pub async fn create(node: &Node) -> Result<(), NodeError> {
		let mut config = node.config.get().await;

		let hostname = match hostname::get() {
			Ok(hostname) => hostname.to_str().unwrap_or_default().to_owned(),
			Err(_) => "unknown".to_owned(),
		};

		let platform = match env::consts::OS {
			"windows" => Platform::Windows,
			"macos" => Platform::MacOS,
			"linux" => Platform::Linux,
			_ => Platform::Unknown,
		};

		let node_data = match node
			.db
			.node()
			.find_unique(node::pub_id::equals(config.node_pub_id.clone()))
			.exec()
			.await?
		{
			Some(node) => node,
			None => {
				node.db
					.node()
					.create(
						node::pub_id::set(config.node_pub_id.clone()),
						node::name::set(hostname.clone()),
						vec![node::platform::set(platform as i32)],
					)
					.exec()
					.await?
			}
		};

		config = node
			.config
			.write(move |mut config| {
				config.node_name = hostname;
				config.node_id = node_data.id;
			})
			.await
			.unwrap();

		Ok(())
	}
}

#[derive(Error, Debug)]
pub enum NodeError {
	#[error("Database error")]
	DatabaseError(#[from] prisma::QueryError),
}