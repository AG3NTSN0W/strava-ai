const map = L.map('map');
L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {attribution: '© OpenStreetMap'}).addTo(map);

const group = L.featureGroup();
pts.forEach(p => {
    // L.circleMarker([p[0],p[1]],{radius:4,fillColor:'hsl('+p[3]+',100%,50%)',fillOpacity:1,stroke:false}).addTo(group);
    L.hotline(p, {
        min: low_count,   // Value corresponding to the first palette color
        max: max_count,  // Value corresponding to the last palette color
        palette: palette,
        weight: 5,
        outlineColor: '#000',
        outlineWidth: 1
    }).addTo(group);
});


group.addTo(map);
if (group.getLayers().length) map.fitBounds(group.getBounds(), {padding: [30, 30]});
else map.setView([0, 0], 2);
const legend = L.control({position: 'bottomright'});
legend.onAdd = function () {
    const d = L.DomUtil.create('div');
    d.style.cssText = 'background:white;padding:8px 12px;border-radius:4px;box-shadow:0 0 4px rgba(0,0,0,.3);font:12px sans-serif';
    d.innerHTML = `<div style=\"margin-bottom:4px\"><b>Frequency</b></div><div style=\"display:flex;align-items:center;gap:6px\"><span>Low</span><div style=\"width:120px;height:14px;border-radius:3px;background:linear-gradient(to right,${frequency_colors})\"></div><span>High</span></div>`;
    return d;
};
legend.addTo(map);