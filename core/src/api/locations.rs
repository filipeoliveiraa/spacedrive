use std::path::PathBuf;

use rspc::Type;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
	encode::THUMBNAIL_CACHE_DIR_NAME,
	invalidate_query,
	library::calculate_statistics,
	prisma::{file_path, location},
	sys::{self, create_location, get_location, scan_location},
};

use super::{LibraryArgs, RouterBuilder};

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct DirectoryWithContents {
	pub directory: file_path::Data,
	pub contents: Vec<file_path::Data>,
}

#[derive(Type, Deserialize)]
pub struct LocationUpdateArgs {
	pub id: i32,
	pub name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Type)]
pub struct GetExplorerDirArgs {
	pub location_id: i32,
	pub path: String,
	pub limit: i32,
}

pub(crate) fn mount() -> RouterBuilder {
	<RouterBuilder>::new()
		.query("get", |ctx, arg: LibraryArgs<()>| async move {
			let (_, library) = arg.get_library(&ctx).await?;

			let locations = library
				.db
				.location()
				.find_many(vec![])
				.with(location::node::fetch())
				.exec()
				.await
				.unwrap();

			Ok(locations)
		})
		.query("getById", |ctx, arg: LibraryArgs<i32>| async move {
			let (id, library) = arg.get_library(&ctx).await?;

			Ok(sys::get_location(&library, id).await.unwrap())
		})
		.query(
			"getExplorerDir",
			|ctx, arg: LibraryArgs<GetExplorerDirArgs>| async move {
				let (args, library) = arg.get_library(&ctx).await?;

				let location = get_location(&library, args.location_id).await.unwrap();

				let directory = library
					.db
					.file_path()
					.find_first(vec![
						file_path::location_id::equals(Some(location.id)),
						file_path::materialized_path::equals(args.path),
						file_path::is_dir::equals(true),
					])
					.exec()
					.await
					.unwrap()
					.unwrap();

				let file_paths = library
					.db
					.file_path()
					.find_many(vec![
						file_path::location_id::equals(Some(location.id)),
						file_path::parent_id::equals(Some(directory.id)),
					])
					.with(file_path::file::fetch())
					.exec()
					.await
					.unwrap();

				Ok(DirectoryWithContents {
					directory: directory.into(),
					contents: file_paths
						.into_iter()
						.map(|mut file_path| {
							if let Some(file) = &mut file_path.file.as_mut().unwrap() {
								// TODO: Use helper function to build this url as as the Rust file loading layer
								let thumb_path = library
									.config()
									.data_directory()
									.join(THUMBNAIL_CACHE_DIR_NAME)
									.join(location.id.to_string())
									.join(&file.cas_id)
									.with_extension("webp");

								file.has_thumbnail = thumb_path.exists();
							}

							file_path
						})
						.collect(),
				})
			},
		)
		.query("getStatistics", |ctx, arg: LibraryArgs<()>| async move {
			let (_, library) = arg.get_library(&ctx).await?;

			Ok(calculate_statistics(&library).await.unwrap())
		})
		.mutation("create", |ctx, arg: LibraryArgs<PathBuf>| async move {
			let (path, library) = arg.get_library(&ctx).await?;

			let location = create_location(&library, &path).await.unwrap();
			scan_location(&library, location.id, path).await;
			Ok(location)
		})
		.mutation(
			"update",
			|ctx, arg: LibraryArgs<LocationUpdateArgs>| async move {
				let (args, library) = arg.get_library(&ctx).await?;

				library
					.db
					.location()
					.find_unique(location::id::equals(args.id))
					.update(vec![location::name::set(args.name)])
					.exec()
					.await
					.unwrap();

				Ok(())
			},
		)
		.mutation("delete", |ctx, arg: LibraryArgs<i32>| async move {
			let (location_id, library) = arg.get_library(&ctx).await?;

			library
				.db
				.file_path()
				.find_many(vec![file_path::location_id::equals(Some(location_id))])
				.delete()
				.exec()
				.await
				.unwrap();

			library
				.db
				.location()
				.find_unique(location::id::equals(location_id))
				.delete()
				.exec()
				.await
				.unwrap();

			invalidate_query!(
				library,
				"locations.get": LibraryArgs<()>,
				LibraryArgs {
					library_id: library.id,
					arg: ()
				}
			);

			info!("Location {} deleted", location_id);

			Ok(())
		})
		.mutation("fullRescan", |ctx, arg: LibraryArgs<i32>| async move {
			let (id, library) = arg.get_library(&ctx).await?;

			sys::scan_location(&library, id, String::new()).await;

			Ok(())
		})
		.mutation("quickRescan", |_, _: LibraryArgs<()>| todo!())
}