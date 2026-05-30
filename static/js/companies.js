/* companies.js — boot ECharts panes on the Companies dashboard.
 *
 * Relies on charts.js exposing window.cernio.bootEchart, and on the
 * matching <script type="application/json" id="data-<kind>"> JSON
 * islands rendered by the maud handler.
 */

(function () {
  'use strict';

  if (!window.cernio || !window.cernio.bootEchart) return;
  const boot = window.cernio.bootEchart;

  // ── Companies × lane donut (large) ──────────────────────────────────────────
  boot('companies-lane-large', (data, theme) => {
    const items = (data.items || []).map((i) => ({
      name: i.label,
      value: i.value,
      itemStyle: { color: i.color },
    }));
    return {
      backgroundColor: theme.bg,
      tooltip: Object.assign({ trigger: 'item' }, theme.tooltip),
      legend: Object.assign({}, theme.legend, {
        orient: 'vertical',
        left: 12,
        top: 'middle',
        right: 'auto',
        itemWidth: 8,
        itemHeight: 8,
      }),
      series: [{
        type: 'pie',
        radius: ['44%', '76%'],
        center: ['62%', '52%'],
        avoidLabelOverlap: true,
        label: {
          color: '#cbd2db',
          fontFamily: 'JetBrains Mono, monospace',
          fontSize: 11,
          formatter: '{b}\n{c}',
        },
        labelLine: { lineStyle: { color: '#2a3038' } },
        itemStyle: { borderColor: '#0a0e13', borderWidth: 1 },
        data: items,
      }],
    };
  });

  // ── ATS / job-board health (horizontal bar) ─────────────────────────────────
  boot('ats-health-companies', (data, theme) => {
    const labels = data.labels || [];
    const values = data.values || [];
    const colors = data.colors || [];
    const series = values.map((v, i) => ({
      value: v,
      itemStyle: { color: colors[i] || '#4f7cff' },
    }));
    return {
      backgroundColor: theme.bg,
      grid: { left: 110, right: 32, top: 12, bottom: 24, containLabel: false },
      tooltip: Object.assign({ trigger: 'axis', axisPointer: { type: 'shadow' } }, theme.tooltip),
      xAxis: {
        type: 'value',
        axisLabel: theme.axisLabel,
        splitLine: theme.splitLine,
        axisLine: theme.axisLine,
      },
      yAxis: {
        type: 'category',
        data: labels,
        inverse: true,
        axisLabel: Object.assign({}, theme.axisLabel, { color: '#cbd2db', fontSize: 11 }),
        axisLine: theme.axisLine,
        axisTick: { show: false },
      },
      series: [{
        type: 'bar',
        data: series,
        barWidth: 16,
        label: {
          show: true,
          position: 'right',
          color: '#cbd2db',
          fontFamily: 'JetBrains Mono, monospace',
          fontSize: 10,
          formatter: '{c}',
        },
      }],
    };
  });

  // ── Geography donut ─────────────────────────────────────────────────────────
  boot('geography', (data, theme) => {
    const palette = ['#7ea8ff', '#7adf9a', '#c39df0', '#ffc94a', '#ff8a5c', '#aab3bf', '#4f5762'];
    const items = (data.items || []).map((i, idx) => ({
      name: i.name,
      value: i.value,
      itemStyle: { color: palette[idx % palette.length] },
    }));
    return {
      backgroundColor: theme.bg,
      tooltip: Object.assign({ trigger: 'item' }, theme.tooltip),
      legend: Object.assign({}, theme.legend, {
        orient: 'vertical',
        left: 8,
        top: 'middle',
        right: 'auto',
        itemWidth: 8,
        itemHeight: 8,
      }),
      series: [{
        type: 'pie',
        radius: ['38%', '68%'],
        center: ['66%', '52%'],
        avoidLabelOverlap: true,
        label: {
          color: '#cbd2db',
          fontFamily: 'JetBrains Mono, monospace',
          fontSize: 10,
          formatter: '{c}',
        },
        labelLine: { length: 6, length2: 6, lineStyle: { color: '#2a3038' } },
        itemStyle: { borderColor: '#0a0e13', borderWidth: 1 },
        data: items,
      }],
    };
  });

  // ── Recently-discovered timeline (vertical bar) ─────────────────────────────
  boot('discovered', (data, theme) => {
    return {
      backgroundColor: theme.bg,
      grid: { left: 44, right: 16, top: 18, bottom: 28, containLabel: true },
      tooltip: Object.assign({ trigger: 'axis', axisPointer: { type: 'shadow' } }, theme.tooltip),
      xAxis: {
        type: 'category',
        data: data.dates || [],
        axisLabel: Object.assign({}, theme.axisLabel, { hideOverlap: true }),
        axisLine: theme.axisLine,
        axisTick: { show: false },
      },
      yAxis: {
        type: 'value',
        axisLabel: theme.axisLabel,
        splitLine: theme.splitLine,
      },
      series: [{
        type: 'bar',
        data: data.values || [],
        barWidth: 10,
        itemStyle: {
          color: '#4f7cff',
          borderRadius: [2, 2, 0, 0],
        },
        emphasis: {
          itemStyle: {
            color: '#7ea8ff',
            shadowBlur: 12,
            shadowColor: 'rgba(79,124,255,0.55)',
          },
        },
      }],
    };
  });
})();
