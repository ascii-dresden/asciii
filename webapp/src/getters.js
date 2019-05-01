const sortProjects = (
    [,{ extras: { sort_index: a } }],
    [,{ extras: { sort_index: b } }]
) => a > b;

const getJson = response => response.json();

const normalizeProjects = raw_projects =>
    Object.entries(raw_projects)
        .sort(sortProjects);

export const getProjectsByYear = year =>
    fetch(`http://localhost:8000/api/full_projects/year/${year}`)
        .then(getJson)
        .then(normalizeProjects);

export const getProjectsWorkingDir = () =>
    fetch(`http://localhost:8000/api/full_projects/workingdir`)
        .then(getJson)
        .then(normalizeProjects);

export const getYears = () =>
    fetch(`http://localhost:8000/api/projects/year`)
        .then(getJson);

export const getVersion = () =>
    fetch(`http://localhost:8000/api/version`)
        .then(getJson);
