tailwind.config = {
    darkMode: "class",
    theme: {
        extend: {
            "colors": {
                "surface-container-lowest": "var(--color-surface-container-lowest)",
                "tertiary-fixed": "var(--color-tertiary-fixed)",
                "tertiary-container": "var(--color-tertiary-container)",
                "secondary-fixed": "var(--color-secondary-fixed)",
                "primary-fixed": "var(--color-primary-fixed)",
                "on-surface-variant": "var(--color-on-surface-variant)",
                "on-secondary-fixed-variant": "var(--color-on-secondary-fixed-variant)",
                "on-primary-fixed-variant": "var(--color-on-primary-fixed-variant)",
                "surface-tint": "var(--color-surface-tint)",
                "outline": "var(--color-outline)",
                "on-secondary": "var(--color-on-secondary)",
                "on-primary-fixed": "var(--color-on-primary-fixed)",
                "on-background": "var(--color-on-background)",
                "surface-container-low": "var(--color-surface-container-low)",
                "surface-container-highest": "var(--color-surface-container-highest)",
                "secondary-fixed-dim": "var(--color-secondary-fixed-dim)",
                "on-secondary-fixed": "var(--color-on-secondary-fixed)",
                "on-error": "var(--color-on-error)",
                "on-surface": "var(--color-on-surface)",
                "primary-container": "var(--color-primary-container)",
                "on-tertiary-fixed-variant": "var(--color-on-tertiary-fixed-variant)",
                "inverse-on-surface": "var(--color-inverse-on-surface)",
                "on-primary-container": "var(--color-on-primary-container)",
                "secondary": "var(--color-secondary)",
                "outline-variant": "var(--color-outline-variant)",
                "surface": "var(--color-surface)",
                "surface-variant": "var(--color-surface-variant)",
                "primary-fixed-dim": "var(--color-primary-fixed-dim)",
                "on-tertiary-fixed": "var(--color-on-tertiary-fixed)",
                "on-primary": "var(--color-on-primary)",
                "surface-container": "var(--color-surface-container)",
                "background": "var(--color-background)",
                "inverse-primary": "var(--color-inverse-primary)",
                "on-secondary-container": "var(--color-on-secondary-container)",
                "on-tertiary-container": "var(--color-on-tertiary-container)",
                "tertiary": "var(--color-tertiary)",
                "surface-dim": "var(--color-surface-dim)",
                "error-container": "var(--color-error-container)",
                "primary": "var(--color-primary)",
                "error": "var(--color-error)",
                "inverse-surface": "var(--color-inverse-surface)",
                "secondary-container": "var(--color-secondary-container)",
                "surface-container-high": "var(--color-surface-container-high)",
                "on-error-container": "var(--color-on-error-container)",
                "on-tertiary": "var(--color-on-tertiary)",
                "tertiary-fixed-dim": "var(--color-tertiary-fixed-dim)",
                "surface-bright": "var(--color-surface-bright)"
            },
            "borderRadius": {
                "DEFAULT": "0.25rem",
                "lg": "0.5rem",
                "xl": "0.75rem",
                "full": "9999px"
            },
            "spacing": {
                "unit": "4px",
                "container-max": "1280px",
                "stack-lg": "32px",
                "stack-md": "16px",
                "stack-sm": "8px",
                "margin-mobile": "16px",
                "margin-desktop": "40px",
                "gutter": "24px"
            },
            "fontFamily": {
                "body-md": ["Inter"],
                "label-sm": ["Geist"],
                "body-lg": ["Inter"],
                "headline-md": ["Hanken Grotesk"],
                "display": ["Hanken Grotesk"],
                "data-mono": ["Geist"],
                "headline-lg": ["Hanken Grotesk"],
                "label-md": ["Geist"]
            },
            "fontSize": {
                "body-md": ["16px", {"lineHeight": "24px", "fontWeight": "400"}],
                "label-sm": ["12px", {"lineHeight": "16px", "letterSpacing": "0.05em", "fontWeight": "600"}],
                "body-lg": ["18px", {"lineHeight": "28px", "fontWeight": "400"}],
                "headline-md": ["24px", {"lineHeight": "32px", "fontWeight": "600"}],
                "display": ["48px", {"lineHeight": "56px", "letterSpacing": "-0.02em", "fontWeight": "800"}],
                "data-mono": ["16px", {"lineHeight": "24px", "fontWeight": "500"}],
                "headline-lg": ["32px", {
                    "lineHeight": "40px",
                    "letterSpacing": "-0.01em",
                    "fontWeight": "700"
                }],
                "label-md": ["14px", {"lineHeight": "20px", "letterSpacing": "0.02em", "fontWeight": "500"}]
            }
        },
    },
}

let map; // declared at the top level

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


const navigate_to = (navigate_to) => {
    set_nav(navigate_to)
    if (ATHLETES && ATHLETES.length === 1) {
        const athlete = ATHLETES[0]

        switch (navigate_to) {
            case "settings":
                htmx.ajax('GET', `/settings?athlete_id=${athlete.id}`, '#main-area');
                break;
            case "activityFeed":
                htmx.ajax('GET', `/athlete?athlete_id=${athlete.id}`, '#main-area');
                break
            case "heatmaps":

                heat_map_afterSettle()

                htmx.ajax('GET', `/heat/map?athlete_id=${athlete.id}`, '#main-area');
                break;
        }
    }

}

function heat_map_afterSettle() {
    const main_area = document.getElementById('main-area');

    main_area.addEventListener('htmx:afterSettle', function(evt) {
        initHeatmap();
    }, { once: true });
}


const set_nav = (id) => {
    let nav_element = document.getElementById("nav-bar");

    for (let element of nav_element.getElementsByTagName("a")) {
        if (id === element.id) {
            element.className = "flex items-center gap-stack-sm px-stack-md py-stack-sm bg-primary-container text-on-primary-container font-bold rounded-lg transition-all font-label-md text-label-md";
        } else {
            element.className = "flex items-center gap-stack-sm px-stack-md py-stack-sm text-secondary hover:bg-surface-container-high rounded-lg transition-all font-label-md text-label-md";
        }
    }


}

const initializeTheme = () => {
    const savedTheme = localStorage.getItem('selectedTheme') || 'light';
    const html = document.documentElement;
    html.classList.remove('light', 'dark');
    html.classList.add(savedTheme);
};

const reGen = (athlete_id, activity_id) => {
    const element = document.getElementById(`activity-${athlete_id}-${activity_id}`);
    htmx.trigger(element, 'click');
}

const settings_request = () => {
    debugger
}

const toggle_info = () => {
    const info_overlay_display = document.getElementById('infoOverlay').style.display
    if (info_overlay_display === 'none') {
        document.getElementById('infoOverlay').style.display = 'block'
    } else {
        document.getElementById('infoOverlay').style.display = 'none'
    }
}


window.onload = () => {
    navigate_to("activityFeed");
    initializeTheme()
}
