import produce from 'immer';
import create from 'zustand';

type LayoutMode = 'list' | 'grid';

export enum ExplorerKind {
	Location,
	Tag,
	Space
}

type ExplorerStore = {
	layoutMode: LayoutMode;
	locationId: number | null; // used by top bar
	showInspector: boolean;
	selectedRowIndex: number;
	multiSelectIndexes: number[];
	contextMenuObjectId: number | null;
	newThumbnails: Record<string, boolean>;
	addNewThumbnail: (cas_id: string) => void;
	selectMore: (indexes: number[]) => void;
	reset: () => void;
	set: (changes: Partial<ExplorerStore>) => void;
};

export const useExplorerStore = create<ExplorerStore>((set) => ({
	layoutMode: 'grid',
	locationId: null,
	showInspector: true,
	selectedRowIndex: 1,
	multiSelectIndexes: [],
	contextMenuObjectId: -1,
	newThumbnails: {},
	addNewThumbnail: (cas_id) =>
		set((state) =>
			produce(state, (draft) => {
				draft.newThumbnails[cas_id] = true;
			})
		),
	selectMore: (indexes) => {
		set((state) =>
			produce(state, (draft) => {
				if (!draft.multiSelectIndexes.length && indexes.length) {
					draft.multiSelectIndexes = [draft.selectedRowIndex, ...indexes];
				} else {
					draft.multiSelectIndexes = [...new Set([...draft.multiSelectIndexes, ...indexes])];
				}
			})
		);
	},
	reset: () => set(() => ({})),
	set: (changes) => set((state) => ({ ...state, ...changes }))
}));
