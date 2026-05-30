/* jobs.js — page-specific chart bootstraps for /jobs.
 *
 * The Jobs page has exactly one ECharts chart: the posting-freshness
 * histogram. Everything else (heatmap, decision funnel, company / title
 * top-lists, jobs table) is pure HTML + CSS driven by the JSON the
 * handler emits via maud.
 */

(function () {
  'use strict';
  if (!window.cernio || !window.cernio.bootEchart) return;

  window.cernio.bootEchart('freshness', (data, theme) => {
    const labels = data.labels || [];
    const values = data.values || [];
    const colors = data.colors || [];
    return {
      backgroundColor: theme.bg,
      grid: { left: 36, right: 16, top: 28, bottom: 28, containLabel: true },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'shadow' },
        backgroundColor: theme.tooltip.backgroundColor,
        borderColor: theme.tooltip.borderColor,
        textStyle: theme.tooltip.textStyle,
        extraCssText: theme.tooltip.extraCssText,
      },
      xAxis: {
        type: 'category',
        data: labels,
        axisLabel: theme.axisLabel,
        axisLine: theme.axisLine,
      },
      yAxis: {
        type: 'value',
        axisLabel: theme.axisLabel,
        splitLine: theme.splitLine,
      },
      series: [{
        type: 'bar',
        data: values.map((v, i) => ({
          value: v,
          itemStyle: { color: colors[i] || '#7ea8ff' },
        })),
        barMaxWidth: 56,
        label: {
          show: true,
          position: 'top',
          color: '#cbd2db',
          fontFamily: 'JetBrains Mono, monospace',
          fontSize: 10,
        },
        emphasis: {
          itemStyle: { shadowBlur: 12, shadowColor: 'rgba(126,168,255,0.6)' },
        },
      }],
    };
  });
})();
