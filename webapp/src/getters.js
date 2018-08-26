const sortProjects = (
    [,{ extras: { sort_index: a } }],
    [,{ extras: { sort_index: b } }]
) => a > b;

const getJson = response => response.json();

const normalizeProjects = raw_projects =>
    Object.entries(raw_projects)
        .sort(sortProjects);

export const getProjectsByYear = year =>
    fetch(`/api/full_projects/year/${year}/`)
        .then(getJson)
        .then(normalizeProjects);

export const getProjectsWorkingDir = () =>
    fetch(`/api/projects/workingdir/`)
        .then(getJson)
        .then(normalizeProjects);

export const getYears = () =>
    fetch(`/api/projects/year`)
        .then(getJson);

export const getVersion = () =>
    fetch(`/api/version`)
        .then(getJson);
