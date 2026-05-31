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

function initMap(layer) {
    console.log("Map initialized, layer:", layer || "route");
    if (map) map.remove();

    map = L.map('map', {
        zoomControl: false,
        dragging: false,
        touchZoom: false,
        doubleClickZoom: false,
        scrollWheelZoom: false,
        boxZoom: false,
        keyboard: false
    });

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {attribution: '© OpenStreetMap'}).addTo(map);

    layer = layer || "route";

    if (layer === "route" || !pts.length) {
        var pointLine = L.polyline(pts, {
            color: '#FC4C02',
            weight: 3,
            opacity: 1,
            smoothFactor: 1
        }).addTo(map);
        if (pts.length) map.fitBounds(pointLine.getBounds());
        return;
    }

    let metricData = [];
    let palette = {};
    let label = "";

    if (layer === "heartrate" && typeof heartrateStream !== 'undefined' && heartrateStream.length) {
        metricData = heartrateStream;
        palette = {0.0: '#ffffff', 0.25: '#ffe0e0', 0.5: '#ffb3b3', 0.75: '#ff6666', 1.0: '#cc0000'};
        label = "Heart Rate";
    } else if (layer === "altitude" && typeof altitudeStream !== 'undefined' && altitudeStream.length) {
        metricData = altitudeStream;
        palette = {0.0: '#006400', 0.25: '#228B22', 0.5: '#ADFF2F', 0.75: '#FFD700', 1.0: '#8B4513'};
        label = "Elevation";
    } else if (layer === "velocity" && typeof velocityStream !== 'undefined' && velocityStream.length) {
        metricData = velocityStream;
        palette = {0.0: '#0000FF', 0.25: '#00BFFF', 0.5: '#00FF00', 0.75: '#FFA500', 1.0: '#FF0000'};
        label = "Speed";
    } else {
        // Fallback to plain route if no data
        var pointLine = L.polyline(pts, {
            color: '#FC4C02',
            weight: 3,
            opacity: 1,
            smoothFactor: 1
        }).addTo(map);
        if (pts.length) map.fitBounds(pointLine.getBounds());
        return;
    }

    // Build hotline data: [[lat, lng, metric], ...]
    const len = Math.min(pts.length, metricData.length);
    const hotlineData = [];
    for (let i = 0; i < len; i++) {
        hotlineData.push([pts[i][0], pts[i][1], metricData[i]]);
    }

    const line = L.hotline(hotlineData, {
        min: Math.min(...metricData.slice(0, len).filter(v => v > 0)),
        max: Math.max(...metricData.slice(0, len)),
        palette: palette,
        weight: 5,
        outlineColor: '#000',
        outlineWidth: 1
    }).addTo(map);

    map.fitBounds(line.getBounds());
}