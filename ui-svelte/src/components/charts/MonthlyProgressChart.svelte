<!-- Derevo (дерево) — A fast, native habit tracker built with Rust & Tauri.
     Copyright (C) 2026  yfloress

     This program is free software: you can redistribute it and/or modify
     it under the terms of the GNU Affero General Public License as
     published by the Free Software Foundation, either version 3 of the
     License, or (at your option) any later version.

     This program is distributed in the hope that it will be useful,
     but WITHOUT ANY WARRANTY; without even the implied warranty of
     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
     GNU Affero General Public License for more details.

     You should have received a copy of the GNU Affero General Public License
     along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>. -->

<script lang="ts">
  import BaseChart from './BaseChart.svelte'
  import { chartLight as L, pick } from '../../lib/charts/theme'
  import { i18n } from '../../lib/stores/i18n.svelte'
  import type { HabitDto } from '../../lib/types/habits'

  interface Props {
    habits: HabitDto[]
    /** Days to plot — today for the current month, full month otherwise. */
    lastDay: number
  }

  let { habits, lastDay }: Props = $props()

  // Daily completion ratio: how many habits were done that day / total habits.
  let rows = $derived.by(() => {
    const total = habits.length
    const out: { day: number, done: number, total: number, ratio: number }[] = []
    for (let d = 1; d <= lastDay; d++) {
      let done = 0
      for (const h of habits) if (h.days[d]) done++
      out.push({ day: d, done, total, ratio: total ? done / total : 0 })
    }
    return out
  })

  let option = $derived.by(() => {
    const accent = pick('#34d399', L.accent)
    const perfect = pick('#fbbf24', '#d97706')
    return {
      backgroundColor: 'transparent',
      grid: { left: 38, right: 16, top: 18, bottom: 24 },
      tooltip: {
        trigger: 'axis',
        backgroundColor: pick('#1a1a1a', L.tooltipBg),
        borderColor: pick('#333', L.tooltipBorder),
        textStyle: { color: pick('#e0e0e0', L.tooltipText), fontSize: 12 },
        formatter: (p: { dataIndex: number }[]) => {
          const r = rows[p[0].dataIndex]
          if (!r) return ''
          const mark = r.ratio >= 1 ? `  ·  ${i18n.t('habits-perfect-day')} ✦` : ''
          return `<b>${r.day}</b> — ${(r.ratio * 100).toFixed(0)}% (${r.done}/${r.total})${mark}`
        },
      },
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: rows.map((r) => String(r.day)),
        axisLine: { lineStyle: { color: pick('#333', L.axisLine) } },
        axisTick: { show: false },
        axisLabel: { color: pick('#888', L.label), fontSize: 10 },
      },
      yAxis: {
        type: 'value',
        min: 0,
        max: 1,
        interval: 0.25,
        axisLine: { show: false },
        splitLine: { lineStyle: { color: pick('#1a1a1a', L.splitLine) } },
        axisLabel: {
          color: pick('#666', L.labelDim), fontSize: 10,
          formatter: (v: number) => `${(v * 100).toFixed(0)}%`,
        },
      },
      series: [
        {
          type: 'line',
          smooth: 0.35,
          showSymbol: false,
          data: rows.map((r) => r.ratio),
          lineStyle: {
            width: 2.5,
            color: accent,
            shadowBlur: 10,
            shadowColor: pick('rgba(52,211,153,0.45)', 'rgba(5,150,105,0.3)'),
          },
          areaStyle: {
            color: {
              type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
              colorStops: [
                { offset: 0, color: pick('rgba(52,211,153,0.35)', 'rgba(5,150,105,0.26)') },
                { offset: 1, color: pick('rgba(52,211,153,0.02)', 'rgba(5,150,105,0.02)') },
              ],
            },
          },
          emphasis: { focus: 'series' },
          z: 2,
        },
        {
          // Glowing dots on perfect days (all habits done).
          type: 'scatter',
          data: rows.map((r) => (r.ratio >= 1 ? 1 : '-')),
          symbolSize: 9,
          itemStyle: {
            color: perfect,
            borderColor: pick('#0a0a0a', '#ffffff'),
            borderWidth: 1.5,
            shadowBlur: 12,
            shadowColor: pick('rgba(251,191,36,0.6)', 'rgba(217,119,6,0.4)'),
          },
          tooltip: { show: false },
          z: 5,
        },
      ],
    }
  })
</script>

<BaseChart {option} height="140px" />

<style></style>
