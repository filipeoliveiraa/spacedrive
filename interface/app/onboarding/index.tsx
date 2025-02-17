import { Navigate, RouteObject } from 'react-router';
import Alpha from './alpha';
import CreatingLibrary from './creating-library';
import NewLibrary from './new-library';
import Privacy from './privacy';
import Start from './start';

export default [
	{
		index: true,
		element: <Navigate to="alpha" replace />
	},
	{ path: 'alpha', element: <Alpha /> },
	{
		element: <Start />,
		path: 'start'
	},
	{
		element: <NewLibrary />,
		path: 'new-library'
	},
	{
		element: <Privacy />,
		path: 'privacy'
	},
	{
		element: <CreatingLibrary />,
		path: 'creating-library'
	}
] satisfies RouteObject[];
