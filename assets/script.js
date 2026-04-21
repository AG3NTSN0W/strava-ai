const themes = {
    dark: '/assets/themes/dark_theme.css',
    light: '/assets/themes/light_theme.css',
};

function connectWithStrava() {
    const hostUrl = window.location.origin;
    const redirectUri = encodeURIComponent(hostUrl + '/exchange_token');
    const stravaAuthUrl = `https://www.strava.com/oauth/authorize?client_id=${STRAVA_CLIENT_ID}&response_type=code&redirect_uri=${redirectUri}&approval_prompt=force&scope=activity:read,activity:write`;

    window.location.href = stravaAuthUrl;
}

const before_request = () => {
    document.getElementById("activity-data").innerHTML = ""
}

const before_request_activity = () => {
    document.getElementById("activity-results").innerHTML = ""
}

const before_request_ai = () => {
    document.getElementById("ai-summary").innerHTML = ""
}

const applyTheme = (themeName) => {
    // Remove existing theme link if it exists
    const existingThemeLink = document.getElementById('theme-link');
    if (existingThemeLink) {
        existingThemeLink.remove();
    }

    // Create new theme link Element
    const themeLink = document.createElement('link');
    themeLink.id = 'theme-link';
    themeLink.rel = 'stylesheet';
    themeLink.href = themes[themeName] ? themes[themeName] : 'light';

    // Add theme link to head
    document.head.appendChild(themeLink);
};

const changeTheme = (event) => {
    const selectedTheme = event.target.value;
    applyTheme(selectedTheme);
    localStorage.setItem('selectedTheme', selectedTheme);
};

const on_load = () => {
    if (ATHLETES && ATHLETES.length === 1) {
        const athlete = ATHLETES[0]
        const element = document.getElementById(`athlete-${athlete['id']}`);
        htmx.trigger(element, 'click');
    }
}

const initializeTheme = () => {
    const savedTheme = localStorage.getItem('selectedTheme') || 'light';
    // Apply the theme
    applyTheme(savedTheme);

    // Set the dropdown value if it exists
    const themeSelector = document.getElementById('theme-selector');
    if (themeSelector) {
        themeSelector.value = savedTheme;
    }
};

const reGen = (athlete_id, activity_id) => {
    const element = document.getElementById(`activity-${athlete_id}-${activity_id}`);
    htmx.trigger(element, 'click');
}

const settings_request = () => {
    debugger
}

window.onload = () => {
    on_load();
    initializeTheme()
}
