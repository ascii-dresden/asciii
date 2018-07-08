import App from './App.html';
import { getYears, getProjectsWorkingDir } from './getters';

const app = new App({
	target: document.body,
	data: {
		years	: [],
		projects: [],
		selectedProject: undefined
	}
});

window.app = app;

getYears().then(years => app.set({ years }));

getProjectsWorkingDir()
	.then(projects => app.set({
		projects,
		selectedYear: "working directory"
	}));

export default app;