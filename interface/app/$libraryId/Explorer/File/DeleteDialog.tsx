import { useLibraryMutation } from '@sd/client';
import { Dialog, UseDialogProps, useDialog } from '@sd/ui';
import { useZodForm } from '@sd/ui/src/forms';

interface Propps extends UseDialogProps {
	location_id: number;
	path_id: number;
}

export default (props: Propps) => {
	const deleteFile = useLibraryMutation('files.deleteFiles');

	return (
		<Dialog
			form={useZodForm()}
			onSubmit={() =>
				deleteFile.mutateAsync({
					location_id: props.location_id,
					path_id: props.path_id
				})
			}
			dialog={useDialog(props)}
			title="Delete a file"
			description="Configure your deletion settings."
			loading={deleteFile.isLoading}
			ctaLabel="Delete"
		>
			<p>TODO: checkbox for "delete all matching files" (only if a file is selected)</p>
		</Dialog>
	);
};