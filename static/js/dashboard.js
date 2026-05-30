/* dashboard.js — Dashboard-tab ECharts bootstraps.
 *
 * Charts are mounted via window.cernio.bootEchart(kind, builder), which
 * locates `#chart-<kind>` + `#data-<kind>` and feeds the builder a parsed
 * JSON object plus the shared ChartTheme.
 *
 * Five charts: pipeline funnel, companies×lane donut, jobs×lane×grade
 * stacked bar with archived overlay, ATS provider horizontal bars, and
 * 30-day activity stacked area.
 */
(function () {
  'use strict';

  if (!window.cernio || !window.cernio.bootEchart) return;
  const boot = window.cernio.bootEchart;

  // ── 1. Pipeline funnel ───────────────────────────────────────────────
  boot('pipeline', (data, theme) => ({
    backgroundColor: theme.bg,
    tooltip: { ...theme.tooltip, trigger: 'item', formatter: '{b}<br/>{c}' },
    series: [{
      type: 'funnel',
      left: '8%',
      right: '8%',
      top: 18,
      bottom: 18,
      width: '84%',
      min: 0,
      // Preserve pipeline order from the data (Companies → Resolved →
      // With-active-jobs → Graded → Decisions). Descending auto-sort would
      // float "Jobs graded" (331) above "With active jobs" (97), which reads
      // backwards for a funnel.
      sort: 'none',
      gap: 2,
      label: {
        show: true,
        position: 'inside',
        color: '#0a0e13',
        fontFamily: 'JetBrains Mono, monospace',
        fontSize: 11,
        formatter: '{b}  {c}',
      },
      itemStyle: { borderColor: '#0a0e13', borderWidth: 1 },
      emphasis: { label: { fontSize: 12 } },
      data: data.items,
      color: ['#4f7cff', '#7ea8ff', '#7adf9a', '#ffc94a', '#ff5c5c'],
    }],
  }));

  // ── 2. Companies × lane donut ────────────────────────────────────────
  boot('companies-lane', (data, theme) => ({
    backgroundColor: theme.bg,
    tooltip: { ...theme.tooltip, trigger: 'item', formatter: '{b}<br/>{c} ({d}%)' },
    legend: { ...theme.legend, orient: 'vertical', left: 8, top: 'middle' },
    series: [{
      type: 'pie',
      radius: ['44%', '76%'],
      center: ['62%', '52%'],
      avoidLabelOverlap: true,
      label: {
        show: true,
        color: '#cbd2db',
        fontFamily: 'JetBrains Mono, monospace',
        fontSize: 10,
        formatter: '{b}\n{c}',
      },
      labelLine: { length: 8, length2: 6, lineStyle: { color: '#2a3038' } },
      itemStyle: { borderColor: '#0a0e13', borderWidth: 1 },
      data: data.items.map(it => ({
        name: it.label,
        value: it.value,
        itemStyle: { color: it.color },
      })),
    }],
  }));

  // ── 3. Jobs × lane × grade with archived overlay ─────────────────────
  boot('jobs-lane-grade', (data, theme) => {
    const series = [];
    // Active series — solid blocks per grade.
    data.grades.forEach((g, gi) => {
      series.push({
        name: g,
        type: 'bar',
        stack: 'total',
        emphasis: { focus: 'series' },
        itemStyle: { color: data.grade_colors[gi] },
        data: data.active.map(row => row[gi]),
      });
    });
    // Archived overlay — hatched, low-opacity, stacked atop the active layer.
    data.grades.forEach((g, gi) => {
      series.push({
        name: g + ' archived',
        type: 'bar',
        stack: 'total',
        emphasis: { focus: 'series' },
        itemStyle: {
          color: data.grade_colors[gi],
          opacity: 0.28,
          decal: {
            symbol: 'line',
            dashArrayY: [2, 2],
            rotation: Math.PI / 4,
            color: 'rgba(255,255,255,0.18)',
          },
        },
        data: data.archived.map(row => row[gi]),
      });
    });
    return {
      backgroundColor: theme.bg,
      grid: { left: 110, right: 24, top: 32, bottom: 28, containLabel: false },
      tooltip: {
        ...theme.tooltip,
        trigger: 'axis',
        axisPointer: { type: 'shadow' },
      },
      legend: {
        ...theme.legend,
        data: data.grades, // only show base-grade legend; archived hidden but selectable
      },
      xAxis: {
        type: 'value',
        axisLabel: theme.axisLabel,
        splitLine: theme.splitLine,
        axisLine: theme.axisLine,
      },
      yAxis: {
        type: 'category',
        data: data.lanes,
        axisLabel: {
          ...theme.axisLabel,
          color: '#cbd2db',
          fontSize: 11,
        },
        axisLine: theme.axisLine,
      },
      series,
    };
  });

  // ── 4. ATS provider health (horizontal bar) ──────────────────────────
  boot('ats-health', (data, theme) => ({
    backgroundColor: theme.bg,
    grid: { left: 120, right: 48, top: 16, bottom: 28, containLabel: false },
    tooltip: { ...theme.tooltip, trigger: 'axis', axisPointer: { type: 'shadow' } },
    xAxis: {
      type: 'value',
      axisLabel: theme.axisLabel,
      splitLine: theme.splitLine,
      axisLine: theme.axisLine,
    },
    yAxis: {
      type: 'category',
      data: data.labels,
      axisLabel: {
        ...theme.axisLabel,
        color: '#cbd2db',
        fontSize: 11,
      },
      axisLine: theme.axisLine,
      inverse: true,
    },
    series: [{
      type: 'bar',
      barWidth: 14,
      label: {
        show: true,
        position: 'right',
        color: '#cbd2db',
        fontFamily: 'JetBrains Mono, monospace',
        fontSize: 10,
      },
      data: data.values.map((v, i) => ({
        value: v,
        itemStyle: { color: data.colors[i] },
      })),
    }],
  }));

  // ── 5. 30-day events by source (stacked area) ────────────────────────
  boot('activity-7d', (data, theme) => ({
    backgroundColor: theme.bg,
    grid: { left: 40, right: 16, top: 28, bottom: 32, containLabel: true },
    tooltip: { ...theme.tooltip, trigger: 'axis' },
    legend: { ...theme.legend, data: Array.from(data.sources) },
    xAxis: {
      type: 'category',
      boundaryGap: false,
      data: data.dates,
      axisLabel: { ...theme.axisLabel, hideOverlap: true },
      axisLine: theme.axisLine,
    },
    yAxis: {
      type: 'value',
      axisLabel: theme.axisLabel,
      splitLine: theme.splitLine,
      axisLine: theme.axisLine,
    },
    series: data.sources.map((src, i) => ({
      name: src,
      type: 'line',
      stack: 'activity',
      smooth: true,
      symbol: 'none',
      lineStyle: { width: 1, color: data.colors[i] },
      itemStyle: { color: data.colors[i] },
      areaStyle: { opacity: 0.6, color: data.colors[i] },
      emphasis: { focus: 'series' },
      data: data.series[i],
    })),
  }));
})();
