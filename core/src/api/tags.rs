use rspc::Type;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
	invalidate_query,
	prisma::{file, tag},
};

use super::{utils::LibraryRequest, RouterBuilder};

#[derive(Type, Deserialize)]
pub struct TagCreateArgs {
	pub name: String,
	pub color: String,
}

#[derive(Type, Deserialize)]
pub struct TagAssignArgs {
	pub file_id: i32,
	pub tag_id: i32,
}

#[derive(Type, Deserialize)]
pub struct TagUpdateArgs {
	pub id: i32,
	pub name: Option<String>,
	pub color: Option<String>,
}

pub(crate) fn mount() -> RouterBuilder {
	RouterBuilder::new()
		.library_query("get", |_, _: (), library| async move {
			Ok(library.db.tag().find_many(vec![]).exec().await?)
		})
		.library_query("getFilesForTag", |_, tag_id: i32, library| async move {
			Ok(library
				.db
				.tag()
				.find_unique(tag::id::equals(tag_id))
				.exec()
				.await?)
		})
		.library_mutation("create", |_, args: TagCreateArgs, library| async move {
			let created_tag = library
				.db
				.tag()
				.create(
					Uuid::new_v4().as_bytes().to_vec(),
					vec![
						tag::name::set(Some(args.name)),
						tag::color::set(Some(args.color)),
					],
				)
				.exec()
				.await?;

			invalidate_query!(library, "tags.get");

			Ok(created_tag)
		})
		.library_mutation("assign", |_, args: TagAssignArgs, library| async move {
			library.db.tag_on_file().create(
				tag::id::equals(args.tag_id),
				file::id::equals(args.file_id),
				vec![],
			);

			Ok(())
		})
		.library_mutation("update", |_, args: TagUpdateArgs, library| async move {
			library
				.db
				.tag()
				.update(
					tag::id::equals(args.id),
					vec![tag::name::set(args.name), tag::color::set(args.color)],
				)
				.exec()
				.await?;

			invalidate_query!(library, "tags.get");

			Ok(())
		})
		.library_mutation("delete", |_, id: i32, library| async move {
			library.db.tag().delete(tag::id::equals(id)).exec().await?;

			invalidate_query!(library, "tags.get");

			Ok(())
		})
}
