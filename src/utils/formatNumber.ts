/**
 * 将大数字压缩为 K / M / B 格式。
 * 保留一位小数，进一法（向上取整）。
 *   950      → "1.0K"
 *   1500     → "1.5K"
 *   999_999  → "1000.0K"
 *   1_000_000→ "1.0M"
 */
export function formatCompact(num: number): string {
  const n = Math.abs(num);
  if (n >= 1_000_000_000) {
    return (Math.ceil(n / 100_000_000) / 10).toFixed(1) + 'B';
  }
  if (n >= 1_000_000) {
    return (Math.ceil(n / 100_000) / 10).toFixed(1) + 'M';
  }
  if (n >= 1_000) {
    return (Math.ceil(n / 100) / 10).toFixed(1) + 'K';
  }
  return num.toLocaleString();
}

/**
 * 格式化金额：保留 4 位小数，带 $ 前缀。
 */
export function formatCost(cost: number): string {
  return '$' + cost.toFixed(4);
}
