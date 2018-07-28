import App from './App.html';
import { getYears, getProjectsWorkingDir, getVersion } from './getters';

const app = new App({
	target: document.body,
	data: {
		years	: [],
		projects: [],
		selectedYear: null,
		selectedProject: null,
		versionInfo: null,

	}
});

window.app = app;

getYears().then(years => app.set({ years: ['working'].concat(years) }));

getProjectsWorkingDir()
	.then(projects => app.set({
		selectedYear: "working",
		projects,
	}));

getVersion()
	.then(versionInfo=> app.set({ versionInfo }));

export default app;