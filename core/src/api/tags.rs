use rspc::{ErrorCode, Type};
use serde::Deserialize;
use tracing::log::info;
use uuid::Uuid;

use crate::{
	api::locations::{ExplorerContext, ExplorerData, ExplorerItem},
	encode::THUMBNAIL_CACHE_DIR_NAME,
	invalidate_query,
	prisma::{file, tag, tag_on_file},
};

use super::{LibraryArgs, RouterBuilder};

#[derive(Type, Deserialize)]
pub struct TagCreateArgs {
	pub name: String,
	pub color: String,
}

#[derive(Debug, Type, Deserialize)]
pub struct TagAssignArgs {
	pub file_id: i32,
	pub tag_id: i32,
	pub unassign: bool,
}

#[derive(Type, Deserialize)]
pub struct TagUpdateArgs {
	pub id: i32,
	pub name: Option<String>,
	pub color: Option<String>,
}

pub(crate) fn mount() -> RouterBuilder {
	RouterBuilder::new()
		.query("getAll", |ctx, arg: LibraryArgs<()>| async move {
			let (_, library) = arg.get_library(&ctx).await?;

			Ok(library.db.tag().find_many(vec![]).exec().await?)
		})
		.query("getExplorerData", |ctx, arg: LibraryArgs<i32>| async move {
			let (tag_id, library) = arg.get_library(&ctx).await?;

			info!("Getting files for tag {}", tag_id);

			let tag = library
				.db
				.tag()
				.find_unique(tag::id::equals(tag_id))
				.exec()
				.await?
				.ok_or_else(|| {
					rspc::Error::new(ErrorCode::NotFound, format!("Tag <id={tag_id}> not found"))
				})?;

			let files: Vec<ExplorerItem> = library
				.db
				.file()
				.find_many(vec![file::tags::some(vec![tag_on_file::tag_id::equals(
					tag_id,
				)])])
				.with(file::paths::fetch(vec![]))
				.exec()
				.await?
				.into_iter()
				.map(|mut file| {
					// sorry brendan
					// grab the first path and tac on the name
					let oldest_path = &file.paths.as_ref().unwrap()[0];
					file.name = Some(oldest_path.name.clone());
					file.extension = oldest_path.extension.clone();
					// a long term fix for this would be to have the indexer give the Object a name and extension, sacrificing its own and only store newly found Path names that differ from the Object name

					let thumb_path = library
						.config()
						.data_directory()
						.join(THUMBNAIL_CACHE_DIR_NAME)
						.join(&file.cas_id)
						.with_extension("webp");

					file.has_thumbnail = thumb_path.exists();

					ExplorerItem::Object(Box::new(file))
				})
				.collect();

			info!("Got files {}", files.len());

			Ok(ExplorerData {
				context: ExplorerContext::Tag(tag),
				items: files,
			})
		})
		.query("getForFile", |ctx, arg: LibraryArgs<i32>| async move {
			let (file_id, library) = arg.get_library(&ctx).await?;

			Ok(library
				.db
				.tag()
				.find_many(vec![tag::tag_files::some(vec![
					tag_on_file::file_id::equals(file_id),
				])])
				.exec()
				.await?)
		})
		.query("get", |ctx, arg: LibraryArgs<i32>| async move {
			let (tag_id, library) = arg.get_library(&ctx).await?;

			Ok(library
				.db
				.tag()
				.find_unique(tag::id::equals(tag_id))
				.exec()
				.await?)
		})
		.mutation(
			"create",
			|ctx, arg: LibraryArgs<TagCreateArgs>| async move {
				let (args, library) = arg.get_library(&ctx).await?;

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

				invalidate_query!(
					library,
					"tags.getAll": LibraryArgs<()>,
					LibraryArgs {
						library_id: library.id,
						arg: ()
					}
				);

				Ok(created_tag)
			},
		)
		.mutation(
			"assign",
			|ctx, arg: LibraryArgs<TagAssignArgs>| async move {
				let (args, library) = arg.get_library(&ctx).await?;

				if args.unassign {
					library
						.db
						.tag_on_file()
						.delete(tag_on_file::tag_id_file_id(args.tag_id, args.file_id))
						.exec()
						.await?;
				} else {
					library
						.db
						.tag_on_file()
						.create(
							tag::id::equals(args.tag_id),
							file::id::equals(args.file_id),
							vec![],
						)
						.exec()
						.await?;
				}

				invalidate_query!(
					library,
					"tags.getForFile": LibraryArgs<i32>,
					LibraryArgs {
						library_id: library.id,
						arg: args.file_id
					}
				);

				Ok(())
			},
		)
		.mutation(
			"update",
			|ctx, arg: LibraryArgs<TagUpdateArgs>| async move {
				let (args, library) = arg.get_library(&ctx).await?;

				library
					.db
					.tag()
					.update(
						tag::id::equals(args.id),
						vec![tag::name::set(args.name), tag::color::set(args.color)],
					)
					.exec()
					.await?;

				invalidate_query!(
					library,
					"tags.getAll": LibraryArgs<()>,
					LibraryArgs {
						library_id: library.id,
						arg: ()
					}
				);

				Ok(())
			},
		)
		.mutation("delete", |ctx, arg: LibraryArgs<i32>| async move {
			let (id, library) = arg.get_library(&ctx).await?;

			library.db.tag().delete(tag::id::equals(id)).exec().await?;

			invalidate_query!(
				library,
				"tags.getAll": LibraryArgs<()>,
				LibraryArgs {
					library_id: library.id,
					arg: ()
				}
			);

			Ok(())
		})
}
