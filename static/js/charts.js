/* charts.js — shared ECharts theme + helpers.
 *
 * Exposes `window.cernio.ChartTheme`, `window.cernio.makeChart`,
 * `window.cernio.readJsonIsland`, and `window.cernio.bootEchart`
 * for per-page chart files to call.
 *
 * Convention: every chart on a page has
 *   <div id="chart-<kind>" class="chart">  (or chart-md/sm/lg/donut)
 *   <script type="application/json" id="data-<kind>">{...}</script>
 * The per-page JS reads the JSON island and calls makeChart.
 */

(function () {
  'use strict';

  if (!window.echarts) {
    // ECharts not loaded yet — exit; per-page scripts will degrade.
    return;
  }

  const ChartTheme = {
    bg: 'transparent',
    grid: { left: 56, right: 24, top: 28, bottom: 32, containLabel: true },
    tooltip: {
      backgroundColor: '#0a0e13',
      borderColor: '#2a3038',
      borderWidth: 1,
      textStyle: { color: '#eef2f8', fontFamily: 'JetBrains Mono, monospace', fontSize: 11 },
      extraCssText: 'box-shadow: 0 8px 32px -8px rgba(79,124,255,0.35); border-radius: 2px;',
    },
    axisLabel: { color: '#7a838f', fontFamily: 'JetBrains Mono, monospace', fontSize: 10 },
    splitLine: { lineStyle: { color: '#1a1f27', type: 'dashed' } },
    axisLine: { lineStyle: { color: '#2a3038' } },
    legend: {
      textStyle: { color: '#aab3bf', fontFamily: 'JetBrains Mono, monospace', fontSize: 10 },
      top: 4, right: 12, icon: 'rect', itemWidth: 10, itemHeight: 10,
    },
  };

  function makeChart(el, opt) {
    const chart = echarts.init(el, null, { renderer: 'canvas' });
    chart.setOption(opt);
    window.addEventListener('resize', () => chart.resize());
    return chart;
  }

  function readJsonIsland(id) {
    const el = document.getElementById(id);
    if (!el) return null;
    try { return JSON.parse(el.textContent); } catch (_) { return null; }
  }

  /**
   * bootEchart(kind, optionBuilder)
   *   kind          — string suffix, used to find #chart-<kind> + #data-<kind>
   *   optionBuilder — (data, theme) => echarts option object
   * Convenience wrapper used by per-page chart files.
   */
  function bootEchart(kind, optionBuilder) {
    const el = document.getElementById('chart-' + kind);
    const data = readJsonIsland('data-' + kind);
    if (!el || !data) return null;
    const opt = optionBuilder(data, ChartTheme);
    if (!opt) return null;
    return makeChart(el, opt);
  }

  window.cernio = window.cernio || {};
  window.cernio.ChartTheme = ChartTheme;
  window.cernio.makeChart = makeChart;
  window.cernio.readJsonIsland = readJsonIsland;
  window.cernio.bootEchart = bootEchart;
})();
