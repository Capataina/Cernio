/* activity.js — Activity-tab chart bootstraps.
 *
 * Three charts:
 *   - activity-30d-full : stacked area, events per source per day (30d)
 *   - event-types       : horizontal bar, event-type frequency (30d)
 *   - velocity          : stacked bar, applied/watching/rejected per day (30d)
 *
 * JSON payloads come from json_island() in the handler.
 */

(function () {
  'use strict';
  if (!window.cernio || !window.cernio.bootEchart) return;
  const { bootEchart } = window.cernio;

  // ── 30-day events by source (stacked area) ──
  bootEchart('activity-30d-full', (data, theme) => {
    const series = (data.sources || []).map((name, i) => ({
      name,
      type: 'line',
      stack: 'events',
      smooth: true,
      showSymbol: false,
      lineStyle: { width: 1.4 },
      areaStyle: { opacity: 0.6 },
      itemStyle: { color: (data.colors || [])[i] || '#7ea8ff' },
      data: (data.series || [])[i] || [],
    }));
    return {
      backgroundColor: theme.bg,
      tooltip: Object.assign({ trigger: 'axis' }, theme.tooltip),
      legend: Object.assign({ data: data.sources || [] }, theme.legend),
      grid: theme.grid,
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: data.dates || [],
        axisLabel: theme.axisLabel,
        axisLine: theme.axisLine,
      },
      yAxis: {
        type: 'value',
        axisLabel: theme.axisLabel,
        splitLine: theme.splitLine,
      },
      series,
    };
  });

  // ── Event-type horizontal bar ──
  bootEchart('event-types', (data, theme) => {
    const labels = data.labels || [];
    const values = data.values || [];
    // ECharts horizontal bars render bottom-up; reverse so the largest sits
    // at the top of the pane.
    const labelsR = labels.slice().reverse();
    const valuesR = values.slice().reverse();
    return {
      backgroundColor: theme.bg,
      tooltip: Object.assign({ trigger: 'axis', axisPointer: { type: 'shadow' } }, theme.tooltip),
      grid: Object.assign({}, theme.grid, { left: 8, right: 32 }),
      xAxis: {
        type: 'value',
        axisLabel: theme.axisLabel,
        splitLine: theme.splitLine,
        axisLine: theme.axisLine,
      },
      yAxis: {
        type: 'category',
        data: labelsR,
        axisLabel: Object.assign({}, theme.axisLabel, { fontSize: 10 }),
        axisLine: theme.axisLine,
      },
      series: [{
        type: 'bar',
        barWidth: 12,
        data: valuesR,
        itemStyle: { color: '#4f7cff', borderRadius: [0, 2, 2, 0] },
        label: {
          show: true,
          position: 'right',
          color: '#aab3bf',
          fontFamily: 'JetBrains Mono, monospace',
          fontSize: 10,
        },
      }],
    };
  });

  // ── Decision velocity (stacked bar) ──
  bootEchart('velocity', (data, theme) => {
    const mkSeries = (name, color, arr) => ({
      name,
      type: 'bar',
      stack: 'decisions',
      data: arr || [],
      itemStyle: { color },
      barMaxWidth: 18,
    });
    return {
      backgroundColor: theme.bg,
      tooltip: Object.assign({ trigger: 'axis', axisPointer: { type: 'shadow' } }, theme.tooltip),
      legend: Object.assign({ data: ['applied', 'watching', 'rejected'] }, theme.legend),
      grid: theme.grid,
      xAxis: {
        type: 'category',
        data: data.dates || [],
        axisLabel: theme.axisLabel,
        axisLine: theme.axisLine,
      },
      yAxis: {
        type: 'value',
        axisLabel: theme.axisLabel,
        splitLine: theme.splitLine,
      },
      series: [
        mkSeries('applied',  '#4ade80', data.applied),
        mkSeries('watching', '#ffc94a', data.watching),
        mkSeries('rejected', '#ff5c5c', data.rejected),
      ],
    };
  });
})();
