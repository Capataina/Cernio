/* decisions.js — boots the Pipeline funnel on /decisions. */
(function () {
  'use strict';
  if (!window.cernio || !window.cernio.bootEchart) return;
  const boot = window.cernio.bootEchart;

  boot('decisions-funnel', (data, theme) => ({
    backgroundColor: theme.bg,
    tooltip: { ...theme.tooltip, trigger: 'item', formatter: '{b}<br/>{c}' },
    series: [{
      type: 'funnel',
      left: '8%', right: '8%', top: 18, bottom: 18,
      width: '84%',
      min: 0,
      sort: 'descending',
      gap: 2,
      label: {
        show: true, position: 'inside',
        color: '#0a0e13',
        fontFamily: 'JetBrains Mono, monospace',
        fontSize: 11,
        formatter: '{b}  {c}',
      },
      itemStyle: { borderColor: '#0a0e13', borderWidth: 1 },
      emphasis: { label: { fontSize: 12 } },
      data: data.items,
      color: ['#7a838f', '#ffc94a', '#4ade80', '#c39df0'],
    }],
  }));
})();
