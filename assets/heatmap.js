function initHeatmap() {
    console.log("heatmap initialized");
    if (map) map.remove();

    map = L.map('map');

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {attribution: '© OpenStreetMap'}).addTo(map);

    const group = L.featureGroup();
    pts.forEach(p => {
        L.hotline(p, {
            min: low_count,
            max: max_count,
            palette: palette,
            weight: 5,
            outlineColor: '#000',
            outlineWidth: 1
        }).addTo(group);
    });

    group.addTo(map);
    if (group.getLayers().length) {
        map.fitBounds(group.getBounds(), {padding: [30, 30]});
    } else {
        map.setView([0, 0], 2);
    }

    const legend = L.control({position: 'bottomright'});
    legend.onAdd = function () {
        const d = L.DomUtil.create('div');
        d.className = 'bg-surface backdrop-blur-md p-4 rounded-lg border border-outline-variant shadow-sm';
        d.innerHTML = `<div style="margin-bottom:4px"><b>Frequency</b></div><div style="display:flex;align-items:center;gap:6px"><span>Low</span><div style="width:120px;height:14px;border-radius:3px;background:linear-gradient(to right,${frequency_colors})"></div><span>High</span></div>`;
        return d;
    };
    legend.addTo(map);
}

function initMap() {
    console.log("Map initialized");
    if (map) map.remove();

    map = L.map('map', {
        zoomControl: false,      // Removes the +/- buttons
        dragging: false,         // Disables click-and-drag panning
        touchZoom: false,        // Disables pinch-to-zoom on mobile
        doubleClickZoom: false,  // Disables double-click zooming
        scrollWheelZoom: false,  // Disables mouse wheel zooming
        boxZoom: false,          // Disables Shift + drag zooming
        keyboard: false          // Disables keyboard navigation
    });

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {attribution: '© OpenStreetMap'}).addTo(map);

    var pointLine = L.polyline(pts, {
        color: '#FC4C02',
        weight: 3,
        opacity: 1,
        smoothFactor: 1
    }).addTo(map);

    map.fitBounds(pointLine.getBounds());
}